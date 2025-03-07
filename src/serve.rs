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

    let get = b"GET / HTTP/1.1\r\n";
    let posts_get = b"GET /posts/";

    if buffer.starts_with(get) {
        let index_path = Path::new("index.html");

        if index_path.exists() {
            let contents = fs::read_to_string(index_path).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                contents.len(),
                contents
            );

            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else {
            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n<h1>404 Not Found: index.html</h1>";
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    } else if buffer.starts_with(posts_get) {
        let req_line = String::from_utf8_lossy(&buffer);
        let filename_start = req_line.find("/posts/").unwrap() + "/posts/".len();
        let filename_end = req_line[filename_start..]
            .find(" ")
            .unwrap_or(req_line.len() - filename_start);
        let filename =
            &req_line[filename_start..filename_start + filename_end].trim_end_matches(".html");

        let posts_path = Path::new("posts");
        let requested_file_path = posts_path.join(format!("{}.html", filename));

        if requested_file_path.exists() {
            let contents = fs::read_to_string(requested_file_path).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                contents.len(),
                contents
            );

            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else {
            let response = format!(
                "HTTP/1.1 404 NOT FOUND\r\n\r\n<h1>404 Not Found: posts/{}.html</h1>",
                filename
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
