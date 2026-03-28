# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Claude Code plugin marketplace — a curated collection of skills, MCP servers, and tools. Plugins are documentation-driven (Markdown with YAML frontmatter) and published as a static HTML book via mdBook + GitHub Pages.

## Build Commands

```bash
./build-book.sh          # Build static HTML book to book-out/
./build-book.sh serve    # Live-reload dev server
./build-book.sh clean    # Remove generated book-src/ and book-out/
```

The build pipeline: `generate-summary.py` walks `plugins/`, generates `book-src/SUMMARY.md`, symlinks all `.md` files into `book-src/`, then runs `mdbook build`.

## Plugin Structure

Each plugin lives under `plugins/` and must be registered in `.claude-plugin/marketplace.json`:

```
plugins/PLUGIN_NAME/
├── .claude-plugin/plugin.json    # name, version, description
└── skills/
    └── SKILL_NAME/
        ├── SKILL.md              # YAML frontmatter (name, description, tools) + content
        └── references/           # Optional supporting docs
```

## Key Files

- `.claude-plugin/marketplace.json` — plugin registry (add new plugins here)
- `plugins/*/. claude-plugin/plugin.json` — per-plugin metadata
- `generate-summary.py` — auto-discovers plugins and generates mdBook SUMMARY.md
- `book.toml` — mdBook config (source: `book-src/`, output: `book-out/`)
- `.github/workflows/deploy-book.yml` — CI deploys to GitHub Pages on push to master

## Architecture Notes

- `book-src/` and `book-out/` are generated (gitignored) — never edit directly
- Build uses symlinks from `book-src/` to `plugins/` source files, so edits to plugin `.md` files are immediately visible during `serve`
- `generate-summary.py` extracts H1 headings from `.md` files for TOC titles, falling back to humanized filenames
- CI is triggered on pushes to `master` (not main)
