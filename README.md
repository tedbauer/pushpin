# Pushpin

A simple static site generator that I use to generate my [personal website](https://www.polarbeardomestication.net/).

## Installation

```
cargo build --release && alias pushpin=$(pwd)/target/release/pushpin
```

## Usage

Create a folder for your site, structured like this:

```
pages/
  notes1.md
  notes2.md
templates/
  index.html
  post.html
style/
  index.css
  post.css
PUSHPIN.yaml
```

Configure `PUSHPIN.yaml` like this:

```
# PUSHPIN.yaml

homepage: home.md

posts:
  - title: 'Post 1'
    date: 2024-04-28
    path: posts/notes1.md
  - title: 'Post 2'
    date: 2024-05-05
    path: posts/notes2.md
  - title: 'Post 3'
    date: 2024-05-05
    path: posts/notes3.md
```

Run `pushpin`. It will generate:

- an `index.html` in the same directory
- a directory of blog post HTML files in `pages/`
