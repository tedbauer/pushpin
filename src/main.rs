use handlebars::Handlebars;
use pulldown_cmark::{CowStr, LinkType, Tag, TagEnd, TextMergeStream};
use pulldown_cmark::{Event, Parser};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use tera::Tera;
use tinytemplate::TinyTemplate;
use yaml_rust::{Yaml, YamlLoader};

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

#[derive(Serialize)]
struct Context {
    content: String,
}

fn push_toc(iter: &mut Vec<Event>, config: Config) -> () {
    for post in config.posts {
        let post_name = post.name;
        iter.push(Event::Start(Tag::List(None)));
        iter.push(Event::Start(Tag::Item));
        iter.push(Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            dest_url: format!("posts/{post_name}.html").into(),
            title: post.title.clone().into(),
            id: post_name.into(),
        }));
        iter.push(Event::Text(post.title.clone().into()));
        iter.push(Event::End(TagEnd::Link));
        //iter.push(Event::Text(CowStr::Boxed(post.title.into())));
        iter.push(Event::End(TagEnd::Item));
        iter.push(Event::End(TagEnd::List(false)));
    }
}

fn main() -> Result<(), std::io::Error> {
    let stylesheet = r#"

    .header {
        background-color: dodgerblue;
    }

    "#;

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

    let template_str = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>My Markdown Content</title>
        <link rel="stylesheet" href="style/stylesheet.css">
        <link
        href="https://fonts.googleapis.com/css2?family=Merriweather:ital,wght@0,300;0,400;0,700;0,900;1,300;1,400;1,700;1,900&display=swap"
        rel="stylesheet">
        <style>

        .container {
            width: 400px;
            margin: 20px;
            padding: 25px;
            border-radius: 20px;
            border-width: 1px;
            border-color: black;
            border-style: solid;
            box-shadow: black 5px 5px;

            font-family: 'Merriweather', serif;
            position: relative;
        }

        .outer {
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 50px;
            position: relative;

        }

        </style>
    </head>
    <body>
        <div class="outer">
        <div class="container">
        {{content}}
        </div>
        </div>
    </body>
    </html>
"#;

    let mut tera = Tera::new("templates/**/*").unwrap();
    tera.add_raw_template("hello", &template_str).unwrap();

    let mut context = tera::Context::new();
    context.insert("content", &html_output);

    let rendered = tera.render("hello", &context).unwrap();

    let mut index = File::create("index.html").unwrap();
    write!(index, "{}", rendered);

    fs::create_dir("style");
    let mut style = File::create("style/stylesheet.css").unwrap();
    write!(style, "{}", stylesheet);

    fs::create_dir("posts");
    for post in config.posts {
        let post_filename = &post.path;
        let post_path = format!("pages/{post_filename}");
        let mut post_file = File::open(post_path);
        let mut post_contents = String::new();
        post_file?.read_to_string(&mut post_contents)?;

        let iterator = TextMergeStream::new(Parser::new(&post_contents));
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, iterator);

        let template_str = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>My Markdown Content</title>
        <link rel="stylesheet" href="style/stylesheet.css">
        <link
        href="https://fonts.googleapis.com/css2?family=Merriweather:ital,wght@0,300;0,400;0,700;0,900;1,300;1,400;1,700;1,900&display=swap"
        rel="stylesheet">
        <style>

        .container {
            width: 400px;
            margin: 20px;
            padding: 25px;
            border-radius: 20px;
            border-width: 1px;
            border-color: black;
            border-style: solid;
            box-shadow: black 5px 5px;

            font-family: 'Merriweather', serif;
            position: relative;
        }

        .outer {
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 50px;
            position: relative;

        }

        </style>
    </head>
    <body>
        <div class="outer">
        <div class="container">
        Back
        {{content}}
        </div>
        </div>
    </body>
    </html>
"#;

        let mut tera = Tera::new("templates/**/*").unwrap();
        tera.add_raw_template("hello2", &template_str).unwrap();

        let mut context = tera::Context::new();
        context.insert("content", &html_output);

        let rendered = tera.render("hello2", &context).unwrap();

        let post_name = post.name;
        let mut post = File::create(format!("posts/{post_name}.html")).unwrap();
        write!(post, "{}", rendered);
    }
    println!("done.");

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
