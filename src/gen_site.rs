use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

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
use serde::Serialize;
use tera::Tera;

#[derive(Deserialize, Debug)]
struct PageMetadata {
    template: String,
}

fn push_toc(iter: &mut Vec<Event>, config: Config) {
    iter.push(Event::Start(Tag::Table(vec![Alignment::Left; 2])));
    for post in config.posts {
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
                r#"<a class="index-link" href="{}">{post_title}</a>"#,
                post.path.replace("md", "html"),
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
    markdown_path: &str,
    tera: &Tera,
    config: &Config,
    template_name: Option<String>,
    title: &str,
    context: &tera::Context,
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

    let mut context = context.clone();
    context.insert("content", &html_content);
    context.insert("title", title);
    context.insert("path", &markdown_path);

    // TODO: if a template is not found, we need a more helpful error message.
    if let Some(template_name) = template_name {
        tera.render(&template_name, &context).map_err(|err| {
            anyhow!("Failed to render Tera template '{template_name}' for '{markdown_path}': {err}")
        })
    } else {
        Ok(html_content)
    }
}

fn write_page(
    markdown_content: &str,
    markdown_path: &str,
    target_path: &PathBuf,
    tera: &Tera,
    config: &Config,
    template_name: Option<String>,
    title: &str,
    context: &tera::Context,
) -> Result<()> {
    let rendered_html = generate_page(
        markdown_content,
        markdown_path,
        tera,
        config,
        template_name,
        title,
        context,
    )?;

    fs::create_dir_all(target_path.parent().unwrap())?;
    let mut target_file = File::create(target_path).unwrap();
    write!(target_file, "{}", rendered_html).map_err(|err| anyhow!("error: {err}"))
}

fn read_metadata(markdown_path: &PathBuf) -> Result<Option<PageMetadata>> {
    let mut markdown_string = String::new();
    let markdown_file = File::open(markdown_path);
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

const INITIAL_INDEX_MD: &str = r#"---
template: index.html
---

# mysite

Welcome to my new site!

[[ListPosts]]
"#;

const INITIAL_POST_MD: &str = r#"---
template: post.html
---

This is an example post.

"#;

const INITIAL_INDEX_TEMPLATE: &str =
    "<!DOCTYPE html><head></head><body>{{content | safe}}</body></html>";
const INITIAL_POST_TEMPLATE: &str =
    "<!DOCTYPE html><head></head><body>{{content | safe}}</body></html>";

const INITIAL_CONFIG: &str = r#"homepage: index.md

posts:
  - title: 'Example post'
    date: 05-05-2024
    path: posts/notes1.md
    name: post1
        "#;

pub(crate) fn initialize(title: Option<String>) -> Result<()> {
    let mut root: String = "".to_string();
    if let Some(t) = title {
        fs::create_dir_all(t.clone())?;
        root = format!("{t}/");
    }
    fs::create_dir_all(format!("{root}pages"))?;
    fs::create_dir_all(format!("{root}pages/posts"))?;
    let mut index_md = File::create(format!("{root}pages/index.md"))?;
    let mut post_md = File::create(format!("{root}pages/posts/notes1.md"))?;

    index_md.write_all(INITIAL_INDEX_MD.as_bytes())?;
    post_md.write_all(INITIAL_POST_MD.as_bytes())?;

    fs::create_dir_all(format!("{root}templates"))?;
    let mut index_template = File::create(format!("{root}templates/index.html"))?;
    let mut post_template = File::create(format!("{root}templates/post.html"))?;
    index_template.write_all(INITIAL_INDEX_TEMPLATE.as_bytes())?;
    post_template.write_all(INITIAL_POST_TEMPLATE.as_bytes())?;

    fs::create_dir_all(format!("{root}style"))?;
    File::create(format!("{root}style/index.css"))?;
    File::create(format!("{root}style/post.css"))?;

    let mut config = File::create(format!("{root}PUSHPIN.yaml"))?;
    config.write_all(INITIAL_CONFIG.as_bytes())?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Page {
    title: String,
    template_path: Option<String>,
    target_path: PathBuf,
    markdown_content: String,
    markdown_path: String,
}

#[derive(Debug, Serialize)]
struct Section {
    title: String,
    pages: Vec<Page>,
    subsections: Vec<Section>,
    order: usize,
}

fn capitalize_string(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c.is_whitespace() {
            capitalize_next = true;
            result.push(c);
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_order_from_pathbuf(path: &PathBuf) -> Option<usize> {
    if let Some(file_name) = path.file_name() {
        if let Some(file_str) = file_name.to_str() {
            if let Some(dash_index) = file_str.find('-') {
                if dash_index > 0 {
                    let prefix = &file_str[..dash_index];
                    if let Ok(number) = prefix.parse::<usize>() {
                        return Some(number);
                    }
                }
            }
        }
    }
    None
}

fn parse_sections(dir: &PathBuf, config: &Config) -> Result<Section> {
    // If the dir starts with an integer followed by a -, assume that the integer is the order.
    let order = parse_order_from_pathbuf(dir).unwrap_or(0);

    let mut section = Section {
        title: dir
            .file_name()
            .ok_or(anyhow!("file name error"))?
            .to_str()
            .map(|s| {
                if let Some(dash_index) = s.find('-') {
                    if dash_index > 0 {
                        let prefix = &s[..dash_index];
                        if prefix.parse::<u32>().is_ok() {
                            return s[dash_index + 1..].to_string();
                        }
                    }
                }
                s.to_string()
            })
            .as_ref()
            .map(|s| s.replace("-", " "))
            .as_ref()
            .map(|s| capitalize_string(s))
            .ok_or(anyhow!("file name error"))?
            .to_string(),
        pages: vec![],
        subsections: vec![],
        order,
    };

    let mut pages = vec![];
    let mut subsections = vec![];

    println!(">>>>");
    println!("Here is the config passed in: {:?}", config);
    println!("<<<<");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let subsection = parse_sections(&path, config)?;
            subsections.push(subsection);
        } else {
            let mut file = File::open(&path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let metadata = read_metadata(&path)?;

            // Check if this is a post defined in config, use config title if so
            let path_str = path.to_str().ok_or(anyhow!("file name error"))?;
            println!("oooooo");
            println!("path_str: {}", path_str);
            println!("oooooo");
            let title = config
                .posts
                .iter()
                .find(|post| format!("pages/{}", post.path) == path_str)
                .map(|post| post.title.clone())
                .unwrap_or_else(|| {
                    path.with_extension("")
                        .file_name()
                        .ok_or(anyhow!("file name error"))
                        .unwrap()
                        .to_str()
                        .map(|s| capitalize_string(&s.replace("-", " ")))
                        .ok_or(anyhow!("file name error"))
                        .unwrap()
                        .to_string()
                });
            let template_path = if let Some(m) = metadata {
                Some(m.template)
            } else {
                None
            };

            let target_path = path.strip_prefix("pages")?;
            let target_path = target_path.with_extension("html");

            let page = Page {
                title,
                template_path,
                target_path,
                markdown_content: content,
                markdown_path: path.to_str().ok_or(anyhow!("file name error"))?.to_string(),
            };

            pages.push(page);
        }
    }

    section.pages = pages;
    section.subsections = subsections;
    section.subsections.sort_by(|a, b| a.order.cmp(&b.order));

    println!("----");
    println!("Parsed section: {:?}", section);
    println!("----");
    Ok(section)
}

fn generate_sections(
    sections: &Section,
    tera: &Tera,
    config: &Config,
    context: &tera::Context,
) -> Result<usize> {
    let mut total = 0;
    for page in &sections.pages {
        write_page(
            &page.markdown_content,
            &page.markdown_path,
            &page.target_path,
            tera,
            config,
            page.template_path.clone(),
            &page.title,
            context,
        )?;
        total += 1;
    }

    for subsection in &sections.subsections {
        total += generate_sections(subsection, tera, config, context)?;
    }

    Ok(total)
}

pub(crate) fn generate(config: &Config) -> Result<usize> {
    let sections = parse_sections(&PathBuf::from("pages"), config)?;
    let tera = Tera::new("templates/*.html")?;
    let mut context = tera::Context::new();
    context.insert("sections", &sections);

    generate_sections(&sections, &tera, config, &context)
}
