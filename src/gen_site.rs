use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use crate::Config;
use anyhow::{anyhow, Result};
use pulldown_cmark::Alignment;
use pulldown_cmark::DefaultBrokenLinkCallback;
use pulldown_cmark::Event;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use pulldown_cmark::TextMergeStream;
use serde::Serialize;
use serde_json::Value; // Added this import
use tera::Tera;

/// Helper to split a document into optional front matter and main content.
/// Returns a tuple of (Option<yaml_string>, main_content_string).
fn split_document(content: &str) -> Result<(Option<&str>, &str)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Ok((None, content));
    }

    // Split the document into at most 3 parts based on the `---` delimiter.
    // 1. An empty string from before the first `---`
    // 2. The YAML front matter
    // 3. The rest of the document (the main markdown content)
    let parts: Vec<&str> = content.splitn(3, "---").collect();

    if parts.len() < 3 {
        // This means we didn't find two `---` delimiters, so it's not valid front matter.
        return Ok((None, content));
    }

    let front_matter = parts[1];
    let main_content = parts[2];

    Ok((Some(front_matter), main_content))
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
            Event::Text(text) if text.as_ref() == "[[ListPosts]]" => {
                push_toc(&mut transformed, config.clone())
            }
            _ => transformed.push(event.clone()),
        }
    }
    Ok(transformed.clone())
}

/// Processes a markdown string into an HTML string.
/// This includes expanding custom macros like [[ListPosts]].
fn process_markdown_content(markdown: &str, config: &Config) -> Result<String> {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(Options::ENABLE_TABLES);
    // NOTE: We don't enable YAML metadata blocks here because the metadata
    // has already been stripped out and processed separately.

    let parser = Parser::new_ext(markdown, options);
    // The TextMergeStream is used to handle our custom macro `[[ListPosts]]`.
    let iterator_with_macros = TextMergeStream::new(parser);
    let events = expand_macros(iterator_with_macros, config)?;

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
    Ok(html_output)
}

/// Renders a single page from its full markdown content to an HTML string.
/// This function handles front matter parsing, content processing, and template rendering.
fn render_page_html(
    full_markdown_content: &str,
    markdown_path: &str,
    tera: &Tera,
    config: &Config,
    // The global_context contains site-wide variables like the section navigation.
    global_context: &tera::Context,
) -> Result<String> {
    // 1. Split document into front matter and main content.
    let (front_matter_str, main_content_str) = split_document(full_markdown_content)?;

    // 2. Parse front matter YAML into a serde_json::Value. This allows us to inspect it.
    let front_matter_value = match front_matter_str {
        Some(yaml_str) => {
            let value: Value = serde_yaml::from_str(yaml_str)
                .map_err(|e| anyhow!("Failed to parse YAML for '{markdown_path}': {e}"))?;
            Some(value)
        }
        None => None,
    };

    // 3. Create the initial Tera context from the parsed value.
    let mut context = match &front_matter_value {
        Some(value) => tera::Context::from_value(value.clone())?,
        None => tera::Context::new(),
    };

    // 4. Process the main markdown body and add it to the context.
    let main_html = process_markdown_content(main_content_str, config)?;
    context.insert("content", &main_html);

    // 5. Process other string values from the front matter as markdown.
    let mut modifications: Vec<(String, String)> = Vec::new();
    if let Some(Value::Object(map)) = &front_matter_value {
        for (key, val) in map {
            // The `template` key is special and should not be processed as markdown.
            if key == "template" {
                continue;
            }
            if let Some(val_str) = val.as_str() {
                let processed_val = process_markdown_content(val_str.trim(), config)?;
                modifications.push((key.clone(), processed_val));
            }
        }
    }

    // Apply the modifications back to the context.
    for (key, processed_val) in modifications {
        context.insert(key, &processed_val);
    }

    // 6. Get template name from the context (it was excluded from markdown processing).
    let template_name = context
        .get("template")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 7. Merge with the global context. The page-specific context will override globals.
    let mut final_context = global_context.clone();
    final_context.extend(context);

    // 8. Render the final HTML.
    if let Some(template) = template_name {
        tera.render(&template, &final_context).map_err(|err| {
            anyhow!("Failed to render Tera template '{template}' for '{markdown_path}': {err:?}")
        })
    } else {
        // If no template is specified, just return the main content's HTML.
        Ok(main_html)
    }
}

/// Takes the rendered HTML and writes it to the final destination file.
fn write_page(
    full_markdown_content: &str, // The full content of the .md file
    markdown_path: &str,
    target_path: &PathBuf,
    tera: &Tera,
    config: &Config,
    global_context: &tera::Context,
    page_title: &str,
) -> Result<()> {
    // We now call our new, powerful render function.
    let mut context_for_page = global_context.clone();
    context_for_page.insert("page_title", page_title);
    let rendered_html = render_page_html(
        full_markdown_content,
        markdown_path,
        tera,
        config,
        &context_for_page,
    )?;

    // Create parent directories if they don't exist.
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut target_file = File::create(target_path)?;
    write!(target_file, "{}", rendered_html)
        .map_err(|err| anyhow!("Failed to write to file: {err}"))
}

const INITIAL_INDEX_MD: &str = r#"---
template: index.html
title: My Site
introduction: |
  This is an **introductory paragraph** rendered from the front matter.
  It's a great place for a summary.
posts_list: "[[ListPosts]]"
my_table: |
  | Header 1 | Header 2 |
  |---|---|
  | Cell 1.1 | Cell 1.2 |
  | Cell 2.1 | Cell 2.2 |
---

Welcome to my new site! This is the main content area.
"#;

const INITIAL_POST_MD: &str = r#"---
template: post.html
title: Example Post
---

This is an example post.

"#;

const INITIAL_INDEX_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <title>{{ title | safe }}</title>
</head>
<body>
  <header>
    <h1>{{ title | safe }}</h1>
    <div class="intro">
      {{ introduction | safe }}
    </div>
  </header>
  <hr>
  <main>
    {{ content | safe }}
  </main>
</body>
</html>"#;
const INITIAL_POST_TEMPLATE: &str =
    "<!DOCTYPE html><head><title>{{ title | safe }}</title></head><body>{{ content | safe }}</body></html>";

const INITIAL_CONFIG: &str = r#"# The homepage is the page that will be rendered at the root of the site.
# It is relative to the `pages` directory.
homepage: index.md

# The `posts` list is used by the [[ListPosts]] macro.
posts:
  - title: 'Example Post'
    date: '2024-05-05' # Use YYYY-MM-DD format
    path: posts/notes1.md
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

/// The Page struct is now just a container for the raw data needed for rendering.
/// The title and template path are parsed from the content during generation.
#[derive(Debug, Serialize)]
struct Page {
    target_path: PathBuf,
    markdown_content: String,
    markdown_path: String,
    title: String,
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

/// Traverses the `pages` directory and builds a tree structure of sections and pages.
fn parse_sections(dir: &PathBuf, config: &Config) -> Result<Section> {
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

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let subsection = parse_sections(&path, config)?;
            subsections.push(subsection);
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let mut file = File::open(&path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let target_path = path.strip_prefix("pages")?;
            let target_path = target_path.with_extension("html");

            let (front_matter_str, _) = split_document(&content)?;
            let front_matter_value = if let Some(yaml_str) = front_matter_str {
                serde_yaml::from_str::<serde_json::Value>(yaml_str).ok()
            } else {
                None
            };

            let title = if let Some(Value::Object(map)) = front_matter_value {
                if let Some(Value::String(t)) = map.get("title") {
                    t.clone()
                } else {
                    capitalize_string(
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .replace("-", " ")
                            .as_str(),
                    )
                }
            } else {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| capitalize_string(s.replace("-", " ").as_str()))
                    .unwrap_or("".to_string())
            };

            let page = Page {
                target_path,
                markdown_content: content,
                markdown_path: path.to_str().ok_or(anyhow!("file name error"))?.to_string(),
                title,
            };

            pages.push(page);
        }
    }

    section.pages = pages;
    section.subsections = subsections;
    section.subsections.sort_by(|a, b| a.order.cmp(&b.order));

    Ok(section)
}

/// Recursively generates the HTML for all pages in all sections.
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
            context, // This is the global context
            &page.title,
        )?;
        total += 1;
    }

    for subsection in &sections.subsections {
        total += generate_sections(subsection, tera, config, context)?;
    }

    Ok(total)
}

/// The main entry point for site generation.
pub(crate) fn generate(config: &Config) -> Result<usize> {
    // Recursively parse the file structure in the `pages` directory.
    let sections = parse_sections(&PathBuf::from("pages"), config)?;

    // Initialize the Tera templating engine. Use `**` for recursive glob.
    let tera =
        Tera::new("templates/**/*.html").map_err(|e| anyhow!("Failed to initialize Tera: {e}"))?;

    // Create a global context and add the site structure to it.
    // This makes the `sections` variable available to all templates for navigation.
    let mut context = tera::Context::new();
    context.insert("sections", &sections);

    // Start the recursive generation process.
    generate_sections(&sections, &tera, config, &context)
}
