use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use crate::Config;
use anyhow::{anyhow, Result};
use pulldown_cmark::Alignment;
use pulldown_cmark::CowStr;
use pulldown_cmark::DefaultBrokenLinkCallback;
use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use pulldown_cmark::TextMergeStream;
use tera::Tera;

fn push_toc(iter: &mut Vec<Event>, config: Config) -> () {
    iter.push(Event::Start(Tag::Table(vec![Alignment::Left; 2])));
    for post in config.posts {
        let post_name = post.name;
        let date_string = post.date.replace("-", "/");

        iter.push(Event::Start(Tag::TableRow));

        iter.push(Event::Start(Tag::TableCell));
        iter.push(Event::Html(format!(
            r#"<div style="color: grey; font-weight: lighter;">{date_string}</div>"#,
        ).into()));
        iter.push(Event::End(TagEnd::TableCell));

        iter.push(Event::Start(Tag::TableCell));
        iter.push(Event::Html(format!(
            r#"<a class="index-link" href="posts/{post_name}.html">{post_title}</a>"#,
            post_name = post_name,
            post_title = post.title
        ).into()));
        iter.push(Event::End(TagEnd::TableCell));

        iter.push(Event::End(TagEnd::TableRow));
    }
    iter.push(Event::End(TagEnd::Table));
}

fn expand_macros<'a>(
    iter: TextMergeStream<'a, Parser<'a, DefaultBrokenLinkCallback>>,
    config: &Config,
) -> Result<Vec<Event<'a>>> {
    let mut transformed: Vec<Event> = Vec::new();
    for event in iter {
        match event {
            Event::Text(CowStr::Boxed(ref text))
                if **text == *<&str as Into<String>>::into("[[ListPosts]]") =>
            {
                push_toc(&mut transformed, config.clone())
            }
            _ => transformed.push(event.clone()),
        }
    }
    Ok(transformed.clone())
}

fn generate_page(
    markdown: &str,
    template_name: &str,
    tera: &Tera,
    config: &Config,
    source_path: &str,
) -> Result<String> {
    let iterator = TextMergeStream::new(Parser::new(&markdown));
    let html_content = expand_macros(iterator, &config).and_then(|events| {
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
        Ok(html_output)
    })?;

    let mut context = tera::Context::new();
    context.insert("content", &html_content);
    context.insert("path", source_path);

    tera.render(template_name, &context)
        .map_err(|err| anyhow!("tera render failed: {err}"))
}

fn write_page(
    markdown_path: &str,
    target_path: &str,
    template_name: &str,
    tera: &Tera,
    config: &Config,
    source_path: &str,
) -> Result<()> {
    let markdown_file = File::open(format!("pages/{markdown_path}"));
    let mut markdown_content = String::new();
    markdown_file?.read_to_string(&mut markdown_content)?;

    let rendered_html = generate_page(&markdown_content, template_name, tera, config, source_path)?;

    let mut target_file = File::create(target_path).unwrap();
    write!(target_file, "{}", rendered_html).map_err(|err| anyhow!("error: {err}"))
}

fn load_templates() -> Result<Tera> {
    let mut tera = Tera::new("templates/**/*")?;

    let index_template = include_str!("templates/index.html.template");
    tera.add_raw_template("index", &index_template)?;

    let post_template = include_str!("templates/post.html.template");
    tera.add_raw_template("post", &post_template)?;

    Ok(tera)
}

pub(crate) fn generate(config: &Config) -> Result<usize> {
    let tera = load_templates()?;

    fs::create_dir_all("style")?;
    fs::create_dir_all("images")?;

    let stylesheet = include_str!("style/stylesheet.css");
    let mut stylesheet_file = File::create("style/stylesheet.css")?;
    write!(stylesheet_file, "{}", stylesheet)?;

    let back_arrow_image = include_bytes!("images/back.png");
    let back_arrow_file = File::create("images/back.png");
    back_arrow_file?.write_all(back_arrow_image)?;

    write_page(
        &config.homepage,
        "index.html",
        "index",
        &tera,
        config,
        &config.homepage,
    )?;
    let mut num_generated_files = 1;

    fs::create_dir_all("posts")?;
    for post in &config.posts {
        let name = &post.name;
        write_page(
            &post.path,
            format!("posts/{name}.html").as_str(),
            "post",
            &tera,
            config,
            &post.path,
        )?;

        num_generated_files += 1;
    }
    Ok(num_generated_files)
}
