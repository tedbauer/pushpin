use pulldown_cmark::{CowStr, Tag, TagEnd, TextMergeStream};
use pulldown_cmark::{Event, Parser};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, Clone)]
struct Post {
    title: String,
    date: String,
    path: String,
}

#[derive(Debug, Clone)]
struct Config {
    homepage: String,
    posts: Vec<Post>,
}

fn push_toc(iter: &mut Vec<Event>, config: Config) -> () {
    for post in config.posts {
        iter.push(Event::Start(Tag::List(None)));
        iter.push(Event::Start(Tag::Item));
        iter.push(Event::Text(CowStr::Boxed(post.title.into())));
        iter.push(Event::End(TagEnd::Item));
        iter.push(Event::End(TagEnd::List(false)));
    }
}

fn main() -> Result<(), std::io::Error> {
    // Read the YAML file
    let mut file = File::open("PUSHPIN.yaml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut yaml_loader = YamlLoader::load_from_str(&contents).unwrap();
    let yaml_doc = &yaml_loader[0];
    let config = parse_config(&yaml_doc);

    let home_filename = &config.homepage;
    let home_path = format!("pages/{home_filename}");
    let mut home_file = File::open(home_path);
    let mut home_contents = String::new();
    home_file?.read_to_string(&mut home_contents)?;

    let iterator = TextMergeStream::new(Parser::new(&home_contents));
    let mut transformed: Vec<Event> = Vec::new();
    for event in iterator {
        match event {
            Event::Text(CowStr::Boxed(ref text))
                if **text == *<&str as Into<String>>::into("[[ListPosts]]") =>
            {
                push_toc(&mut transformed, config.clone())
            }
            _ => transformed.push(event),
        }
    }

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, transformed.into_iter());

    let mut index = File::create("index.html").unwrap();
    write!(index, "{}", html_output);

    Ok(())
}

fn parse_config(yaml_doc: &Yaml) -> Config {
    let homepage = yaml_doc["homepage"].as_str().unwrap().to_string();
    let mut posts = Vec::new();
    let posts_node = &yaml_doc["posts"];
    for post in posts_node.as_vec().unwrap() {
        let title = post["title"].as_str().unwrap().to_string();
        let date = post["date"].as_str().unwrap().to_string();
        let path = post["path"].as_str().unwrap().to_string();
        posts.push(Post { title, date, path });
    }
    Config { homepage, posts }
}
