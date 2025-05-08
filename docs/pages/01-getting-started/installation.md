---
template: installation.html
---

# Installation

Pushpin is written in Rust. Install it by cloning the repo, building it, and creating an alias.

```
git clone https://github.com/tedbauer/pushpin
cd pushpin
cargo build --release && alias pushpin=$(pwd)/target/release/pushpin
```
