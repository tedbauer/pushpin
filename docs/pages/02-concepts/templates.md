---
template: templates.html
---

# Templates

Templates live in the `templates/` directory. They're [Tera](https://keats.github.io/tera/docs/) templates. The organization of the templates is arbitrary. Any number of Pages can [link to a template](pages.html).

## Access data in templates

A [Page can associate with a single Template](pages.html). Template access Page content with the `content` Tera variable. Escape the content with `content | safe`. For example, a trivial template could look like this:

```
<!DOCTYPE html>
<html>
  <head></head>
  <body>
  {{ content | safe}}
  </body>
</html>
```

[Section data](sections.html) is available globally to any template.
