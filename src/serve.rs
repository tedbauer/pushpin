use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;

pub(crate) fn serve() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("📌 local server available at http://127.0.0.1:7878");
    println!("\nType Ctrl+C to stop.");
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let req_line = String::from_utf8_lossy(&buffer);

    if req_line.starts_with("GET / HTTP/1.1") {
        serve_file(&mut stream, "index.html", "text/html; charset=utf-8");
    } else if req_line.starts_with("GET /") {
        // Extract the requested path
        let start_index = req_line.find('/').unwrap() + 1;
        let end_index = req_line[start_index..]
            .find(' ')
            .unwrap_or(req_line.len() - start_index)
            + start_index;
        let requested_path = &req_line[start_index..end_index];

        if requested_path.is_empty() {
            // This case should ideally not be reached if the first condition works
            return;
        }

        let path = Path::new(requested_path);
        if path.exists() {
            if path.is_file() {
                let extension = path.extension().and_then(|s| s.to_str());
                match extension {
                    Some("html") => {
                        serve_file(&mut stream, requested_path, "text/html; charset=utf-8")
                    }
                    Some("css") => serve_file(&mut stream, requested_path, "text/css"),
                    Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("svg") => {
                        serve_file_binary(&mut stream, requested_path);
                    }
                    _ => {
                        serve_file_binary(&mut stream, requested_path);
                    }
                }
            } else {
                let response = format!(
                    "HTTP/1.1 404 NOT FOUND\r\n\r\n<h1>404 Not Found: {}</h1>",
                    requested_path
                );
                stream.write_all(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        } else {
            let response = format!(
                "HTTP/1.1 404 NOT FOUND\r\n\r\n<h1>404 Not Found: {}</h1>",
                requested_path
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    } else {
        let response = "HTTP/1.1 400 BAD REQUEST\r\n\r\n<h1>400 Bad Request</h1>";
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn serve_file(stream: &mut TcpStream, filepath: &str, content_type: &str) {
    let path = Path::new(filepath);
    if path.exists() {
        let contents = fs::read_to_string(path).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type,
            contents.len(),
            contents
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\n\r\n<h1>404 Not Found: {}</h1>",
            filepath
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn serve_file_binary(stream: &mut TcpStream, filepath: &str) {
    let path = Path::new(filepath);
    if path.exists() {
        let contents = fs::read(path).unwrap();
        let content_type = match path.extension().and_then(|s| s.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            _ => "application/octet-stream", // Default binary type
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            content_type,
            contents.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(&contents).unwrap();
        stream.flush().unwrap();
    } else {
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\n\r\n<h1>404 Not Found: {}</h1>",
            filepath
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
