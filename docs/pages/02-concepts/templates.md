---
template: templates.html
---

# Templates

Templates live in the `templates/` directory. They're [Tera](https://keats.github.io/tera/docs/) templates. The organization of the templates is arbitrary. Any number of Pages can [link to a template](pages.html). A [Page can associate with a single Template](pages.html).

## Access data in templates

These are template variables available:
- `content`, which contains HTML converted from the Markdown of the available Page. It needs to be escaped; when you refer to it, write `content | safe`.
- `section`, which provides [information about the Section containing this Page](sections.html).

A trivial template could look like this:

```
<!DOCTYPE html>
<html>
  <head></head>
  <body>
  {{ content | safe}}
  </body>
</html>
```
