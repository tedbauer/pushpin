use pulldown_cmark::Event;
use pulldown_cmark::{LinkType, Tag, TagEnd};

use std::fs::File;
use std::io::Read;

use yaml_rust::{Yaml, YamlLoader};

mod gen_site;

#[derive(Debug, Clone)]
struct Post {
    title: String,
    date: String,
    path: String,
    name: String,
}

#[derive(Debug, Clone)]
struct Config {
    homepage: String,
    posts: Vec<Post>,
}

fn parse_config(yaml_doc: &Yaml) -> Config {
    let homepage = yaml_doc["homepage"].as_str().unwrap().to_string();
    let mut posts = Vec::new();
    let posts_node = &yaml_doc["posts"];
    for post in posts_node.as_vec().unwrap() {
        let title = post["title"].as_str().unwrap().to_string();
        let date = post["date"].as_str().unwrap().to_string();
        let path = post["path"].as_str().unwrap().to_string();
        let name = post["name"].as_str().unwrap().to_string();
        posts.push(Post {
            title,
            date,
            path,
            name,
        });
    }
    Config { homepage, posts }
}

fn main() -> Result<(), std::io::Error> {
    let mut config_file = File::open("PUSHPIN.yaml")?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;

    let yaml_loader = YamlLoader::load_from_str(&contents).unwrap();
    let yaml_doc = &yaml_loader[0];
    let config = parse_config(&yaml_doc);

    match gen_site::generate(&config) {
        Ok(num_files_generated) => println!("Success: generated {num_files_generated} files."),
        Err(err) => println!("Error: {err}"),
    };

    Ok(())
}