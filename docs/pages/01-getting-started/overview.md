---
template: overview.html
---

# Overview

Pushpin is a static site generator focused on simplicity. Write site content in Markdown, style with [Tera](https://keats.github.io/tera/docs/) HTML templates, and Pushpin compiles the site to HTML. There are tons of similar  and inspiring static site generators, to name a few:

- [Zola](https://www.getzola.org/)
- [Jekyll](https://jekyllrb.com/)
- [Hugo](https://gohugo.io/)
- [mdBook](https://rust-lang.github.io/mdBook/)
- [Pelican](https://getpelican.com/)

## Quick tour

We'll walk through creating a new site, building it, and examining its structure. Then we'll add a new page and template.

### Create and examine a new site

After [installing Pushpin](installation.html), generate a site:

```
$ pushpin init my-new-site
📌 success: initialized new site 'my-new-site'
```

`my-new-site` will contain this directory structure:

```
- pages/
    - index.md
    - posts/example_post.md
- templates/
    - index.html
    - post.html
- style/
    - style.css
```

- `pages` contains the content of your site in Markdown files. Read more about the directory structure in the [Pages page](../02-concepts/pages.html).
- `templates` contains [Tera](https://keats.github.io/tera/docs/) HTML templates that your content will be rendered into. See more information in [Templates](../02-concepts/templates.html).
- `style` contains CSS that your templates can load.

In `my-new-site`, generate the site and serve it with a local server:

```
$ cd my-new-site
$ pushpin serve
📌 success: generated site
📌 local server available at http://127.0.0.1:7878

Type Ctrl+C to stop.
```

Pushpin generates HTML files for each Markdown file in `pages/` in the site root directory. Similar to SSGs like Zola, the directory structure of the output is based on the structure of the Markdown files.

```md
- pages/
    - index.md
    - posts/example-post.md
- templates/
    - index.html
    - post.html
- style/
    - style.css

# Newly generated:
- index.html
- posts/
  - example-post.html
```

This is the primary local development flow. When you [deploy](../03-deployment/gitHub-pages.html), you'll likely use `pushpin generate` to generate the site files, instead of `pushpin serve`.

### Add a new page and template

Create a new directory in `pages/` called `example-section`, and add a Markdown file called `example-page.md` inside. Add a new template for it called `page.html`:

```
- pages/
    - index.md
    - posts/example_post.md
    - example-section/ # New section
      - example-page.md # New page in the new section
- templates/
    - index.html
    - post.html
    - page.html # New template for example-page.md
- style/
    - style.css
- index.html
- pages/
  - example-post.html
```

By adding a directory in `pages/`, we have created a [Section](../02-concepts/sections.html), one way to organize site pages.

Write some content in `example-page.md`. Associate it with its template with Frontmatter:

```
---
template: page.html
---

# Example page header

This is an example page.
```

Add this code for the template:

```html
<!DOCTYPE html>
<html>
  <head></head>
  <body>
  {{ content | safe}}
  </body>
</html>
```

The Markdown content is compiled to HTML, and then inserted into templates via `content | safe`.

Finally, navigate to `http://127.0.0.1:7878/example-section/example-page.html` to see the generated page.

We've created a site, examined its structure, and we added a new page. This is the primary development loop. To learn more about each concept, check out the rest of the Concepts, as well as Deployment.
