---
template: sections.html
---

# Sections

Sections are logical organizations of a site. They are inferred from the Markdown file structure in `<site-root>/pages/`. For example,

```
- site-root/
  - pages/
    - section-1
      - page-1a.md
      - page-1b.md
    - section-2
      - page-2a.md
      - page-2b.md
```

Will create two sections: `Section 1` and `Section 2`. The title is the subdirectory name, with `-`s converted to spaces, and the resulting string uppercased.

## Access section data in templates

Section structure is available globally to the Tera templates in `templates/`. It's available as a Tera variable called `section`, as this type:

```rust
#[derive(Debug, Serialize)]
struct Page {
    title: String,                 // The name of the file, with `-`s converted to spaces, and uppercased.
    template_path: Option<String>, // Path to associated template, if specified in Frontmatter.
    target_path: PathBuf,          // The path of the generated HTML file, relative to the site root dir.
    markdown_content: String,      // The Markdown content of the page.
    markdown_path: String,         // The path to the Markdown file, relative to the site dir.
}

#[derive(Debug, Serialize)]
struct Section {
    title: String,             // The title of the section, with `-`s converted to spaces, and uppercased.
    pages: Vec<Page>,          // The pages contained in the section.
    subsections: Vec<Section>, // Any sections contained within the section.
    order: usize,              // The order that the Section shows up in `section`, inferred from the filename.
}
```

 Access it to for example create a navbar or table of contents:

```

<div class="nav">
    {% macro render_section(section, selected) %}
    <h2>{{ section.title }}</h2>

    {% for page in section.pages %}
    <a href="../{{ page.target_path }}">
        <h3 {% if page.title==selected %}class="selected" {% endif %}>{{page.title}}</h3>
    </a>
    {% endfor %}

    {% if section.subsections %}
    <ul>
        {% for subsection in section.subsections %}
        {{ self::render_section(section=subsection) }}
        {% endfor %}
    </ul>
    {% endif %}

    {% endmacro render_section %}

    {% for subsection in sections.subsections %}
    {{ self::render_section(section=subsection, selected="Title Of Selected Page") }}
    {% endfor %}
</div>
```

## Specify section ordering

The order that a `Section` appears in `subsections` can be specified in the directory names, for example `01-example-ordered-section/`. Without an order at the beginning, the order will be alphabetical.
