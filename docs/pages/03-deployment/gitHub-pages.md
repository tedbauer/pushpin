---
template: github-pages.html
---

# Deploy on GitHub Pages

GitHub lets you serve HTML file pages from a configurable branch and directory in the repository. Check out GitHub's [instructions for setting that up](https://docs.github.com/en/pages/quickstart). Note that by default, GitHub Pages sets up a workflow that runs Jekyll and publishes that; we want to disable that by creating an empty file called `.nojekyll` in the website root, as well as in `docs/` if you publish your site in that directory.

To deploy a site with Pushpin and GitHub pages, you can set up some automation for each commit to your site's repo:

1. Checkout a publish branch that GitHub pages serves HTML from.
2. Run Pushpin in the site root directory to generate HTML pages.

Here's an example GitHub action configuration to deploy a site:

```
name: Generate site and push to GitHub pages branch

on:
  push:
    branches: [ "master" ]
  pull_request:
    branches: [ "master" ]

jobs:
  generate_and_commit:
    runs-on: ubuntu-latest

    permissions:
      contents: write

    steps:
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Checkout website repo
        uses: actions/checkout@v3
        with:
          fetch-depth: 0
          ref: master

      - name: Clone pushpin
        uses: actions/checkout@v3
        with:
          repository: tedbauer/pushpin
          ref: master
          path: cloned-repo

     - name: Build, install, and run pushpin
        working-directory: cloned-repo
        run: |
          cargo build --release
          PUSHPIN_PATH=$(pwd)/target/release/pushpin
          echo "PUSHPIN_PATH=$PUSHPIN_PATH" >> $GITHUB_ENV
          cd ..
          $PUSHPIN_PATH generate

      - name: Commit changes and push
        run: |
          git config user.name github-actions
          git config user.email github-actions@github.com
          git add .
          git checkout -b gh-pages
          git commit -m "Automated file generation"
          git push https://${{ secrets.ACCESS_TOKEN }}@github.com/<your-username>/<your-repo-link> HEAD:gh-pages --force
```
