# Pushpin

A simple static site generator.

## Installation

```
cargo build --release && alias pushpin=$(pwd)/target/release/pushpin
```

## Usage

Configure a site with a `PUSHPIN.yaml`:

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

In the same directory, just run `pushpin`.
