---
template: cli.html
---

# Pushpin CLI

The CLI supports these commands:

- `pushpin init <sitename>`: generate skeleton files for a brand new site in a new directory, `<sitename>`.
  - You can omit `<sitename>`, and the site files will be created in the current working directory.
- `pushpin serve [--watch]`: build the site and serve it with a local webserver. You'll use this for local development.
  - If you pass the `--watch` flag, Pushpin will automatically re-run when it detects updates to content or templates.
- `pushpin generate`: build the site. You'll use this when deploying.
