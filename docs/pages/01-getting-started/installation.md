---
template: installation.html
---

# Installation

Pushpin is written in Rust. Install it, then clone the repo, build it, and create an alias.

```
git clone https://github.com/tedbauer/pushpin
cd pushpin
cargo build --release && alias pushpin=$(pwd)/target/release/pushpin
```
