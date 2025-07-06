# Project: Pushpin

Pushpin is a static site generator written in Rust. It takes Markdown files and templates to generate a static HTML website.

## Key Technologies

* **Rust**: The core language for the application logic.
* **Tera**: A flexible and powerful template engine for Rust, used for rendering HTML.
* **Pulldown-cmark**: A Markdown parser used to convert Markdown content into HTML.
* **Serde**: A serialization/deserialization framework for Rust, used with `serde_yaml` and `serde_json` for configuration and data handling.
* **Clap**: A command-line argument parser, used for handling CLI commands like `gen`, `serve`, and `watch`.
* **Notify**: A cross-platform filesystem notification library, used for watching file changes in `serve` mode.

## Project Structure

* `Cargo.toml`: Defines project dependencies and metadata.
* `Cargo.lock`: Records the exact versions of dependencies.
* `src/`: Contains the main Rust source code.
  * `src/main.rs`: The primary entry point for the application, handling command-line arguments and dispatching to other modules.
  * `src/gen_site.rs`: Contains logic for generating the static site from source files and templates.
  * `src/serve.rs`: Implements the local development server functionality.
  * `src/watcher.rs`: Handles watching for file system changes to trigger site regeneration.
* `docs/`: This directory holds all the source content for documentation of the static site generator itself. Pushpin generates docs for it, published on polarbeardomestication.net/pushpin.
  * `docs/PUSHPIN.yaml`: The main configuration file for the site, defining pages, posts, and other metadata.
  * `docs/pages/`: Contains Markdown files for individual pages.
  * `docs/templates/`: Stores Tera template files (`.html` files) that define the layout and structure of the generated pages.
  * `docs/style/`: Contains CSS files for styling the website.
  * `docs/images/`: Stores image assets.

## Common Tasks & Codepointers

Here are some common tasks you might perform and how Gemini can assist, along with relevant file paths:

* **Build the SSG and create an alias for it:**
  * Command: `cargo build --release && alias pushpin=$(pwd)/target/release/pushpin`
