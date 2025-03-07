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
use pulldown_cmark::MetadataBlockKind;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use pulldown_cmark::TextMergeStream;
use serde::Deserialize;
use tera::Tera;

#[derive(Deserialize, Debug)]
struct PageMetadata {
    template: String,
}

fn push_toc(iter: &mut Vec<Event>, config: Config) {
    iter.push(Event::Start(Tag::Table(vec![Alignment::Left; 2])));
    for post in config.posts {
        let post_name = post.name;
        let date_string = post.date.replace("-", "/");

        iter.push(Event::Start(Tag::TableRow));

        iter.push(Event::Start(Tag::TableCell));
        iter.push(Event::Html(
            format!(r#"<div style="color: grey; font-weight: lighter;">{date_string}</div>"#,)
                .into(),
        ));
        iter.push(Event::End(TagEnd::TableCell));

        iter.push(Event::Start(Tag::TableCell));
        iter.push(Event::Html(
            format!(
                r#"<a class="index-link" href="posts/{post_name}.html">{post_title}</a>"#,
                post_name = post_name,
                post_title = post.title
            )
            .into(),
        ));
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
    tera: &Tera,
    config: &Config,
    source_path: &str,
    title: &str,
) -> Result<String> {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let iterator = TextMergeStream::new(Parser::new_ext(markdown, options));
    let html_content = expand_macros(iterator, config).map(|events| {
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
        html_output
    })?;

    let mut context = tera::Context::new();
    context.insert("content", &html_content);
    context.insert("title", title);
    context.insert("path", source_path);

    tera.render(source_path, &context)
        .map_err(|err| anyhow!("tera render failed: {err}"))
}

fn write_page(
    markdown_path: &str,
    target_path: &str,
    tera: &Tera,
    config: &Config,
    source_path: &str,
    title: &str,
) -> Result<()> {
    let markdown_file = File::open(format!("pages/{markdown_path}"));
    let mut markdown_content = String::new();
    markdown_file?.read_to_string(&mut markdown_content)?;

    let rendered_html = generate_page(&markdown_content, tera, config, source_path, title)?;

    let mut target_file = File::create(target_path).unwrap();
    write!(target_file, "{}", rendered_html).map_err(|err| anyhow!("error: {err}"))
}

fn read_metadata(markdown_path: &str) -> Result<Option<PageMetadata>> {
    let mut markdown_string = String::new();
    println!("tryna open {:?}", format!("pages/{markdown_path}"));
    let markdown_file = File::open(format!("pages/{markdown_path}"));
    let _ = markdown_file?.read_to_string(&mut markdown_string);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = Parser::new_ext(&markdown_string, options);

    let mut in_metadata = false;
    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = true;
            }
            Event::Text(text) => {
                if in_metadata {
                    let metadata = serde_yaml::from_str::<PageMetadata>(&text)?;
                    return Ok(Some(metadata));
                }
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => return Ok(None),
            _ => continue,
        }
    }

    Ok(None)
}

fn load_templates(config: &Config) -> Result<Tera> {
    let mut pages_metadata: Vec<(String, PageMetadata)> = Vec::new();

    let homepage_metadata = read_metadata(&config.homepage)?;
    if let Some(metadata) = homepage_metadata {
        pages_metadata.push((config.homepage.clone(), metadata));
    }

    for post in &config.posts {
        if let Some(metadata) = read_metadata(&post.path)? {
            pages_metadata.push((post.path.clone(), metadata));
        }
    }

    let mut tera = Tera::default();
    for page_metadata in pages_metadata {
        let mut template = String::new();
        println!("tryna open {:?}", page_metadata.1.template);
        let mut template_file = File::open(format!("templates/{}", page_metadata.1.template))?;
        template_file.read_to_string(&mut template)?;

        tera.add_raw_template(page_metadata.0.as_str(), template.as_str())?;
    }

    Ok(tera)
}

pub(crate) fn generate(config: &Config) -> Result<usize> {
    let tera = load_templates(config)?;

    write_page(
        &config.homepage,
        "index.html",
        &tera,
        config,
        &config.homepage,
        "Theodore Bauer",
    )?;
    let mut num_generated_files = 1;

    fs::create_dir_all("posts")?;
    for post in &config.posts {
        let name = &post.name;
        write_page(
            &post.path,
            format!("posts/{name}.html").as_str(),
            &tera,
            config,
            &post.path,
            &post.title,
        )?;

        num_generated_files += 1;
    }
    Ok(num_generated_files)
}
