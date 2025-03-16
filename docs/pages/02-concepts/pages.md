---
template: pages.html
---

# Pages

Pages live in `<site-root>/pages/`, and are Markdown files containing the content of your site.

## Page directory structure

For each page in `pages/`, when you build the site, Pushpin generates an HTML file with the same directory structure, with the site as the root directory. The filename is the same, except the extension is changed to `.html`.

For example, this `pages` setup:

```
- site-root/
  - pages/
    - index.md
    - baz.md
    - foo/
      - foo-page.md
    - bar/
      - bar-page.md
```

After running `pushpin generate` or `pushpin serve`, these files will be generated:

```
- site-root/
  - pages/
    - index.md
    - baz.md
    - foo/
      - foo-page.md
    - bar/
      - bar-page.md

  # Generated files
  - index.html
  - baz.html
    - foo/
      - foo-page.html
    - bar/
      - bar-page.html
```

Note that this setup creates two [Sections](sections.html) named `Foo` and `Bar`.

The resulting HTML paths can be used in Markdown links. For example, `foo-page.md` could link to `index.md` like this:

```
# foo-page.md

This is an [example link to the homepage](../index.html).
```

## Associate a page with a template

Link together a template for a Markdown file with Frontmatter:

```
# index.md
---
template: index.html
---

# Example home page.
```

The path is relative to the `templates/` directory. See [Templates](../02-concepts/templates.html) for more information.
