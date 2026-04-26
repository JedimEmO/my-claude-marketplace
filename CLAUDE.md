# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A shared Claude Code and Codex plugin marketplace - a curated collection of skills, MCP servers, and tools. Plugins are documentation-driven (Markdown with YAML frontmatter) and published as a static HTML book via mdBook + GitHub Pages.

## Build Commands

```bash
./build-book.sh          # Build static HTML book to book-out/
./build-book.sh serve    # Live-reload dev server
./build-book.sh clean    # Remove generated book-src/ and book-out/
```

The build pipeline: `generate-summary.py` walks `plugins/`, generates `book-src/SUMMARY.md`, symlinks all `.md` files into `book-src/`, then runs `mdbook build`.

## Plugin Structure

Each plugin lives under `plugins/` and must be registered for both Claude Code and Codex:

```
plugins/PLUGIN_NAME/
├── .claude-plugin/plugin.json    # Claude name, version, description
├── .codex-plugin/plugin.json     # Codex manifest; points at ./skills/
└── skills/
    └── SKILL_NAME/
        ├── SKILL.md              # YAML frontmatter (name, description, tools) + content
        └── references/           # Optional supporting docs
```

The `skills/`, `references/`, templates, scripts, and other content files are shared source of truth. Do not duplicate skill content for Codex.

## Dual Marketplace Metadata

When adding, renaming, moving, or changing plugin install-facing metadata, update both platform surfaces:

- `.claude-plugin/marketplace.json` - Claude marketplace registry
- `.agents/plugins/marketplace.json` - Codex marketplace registry
- `plugins/*/.claude-plugin/plugin.json` - Claude per-plugin metadata
- `plugins/*/.codex-plugin/plugin.json` - Codex per-plugin metadata

Codex manifests should point at shared skills with `"skills": "./skills/"`. If a plugin includes plugin-local MCP config in `.mcp.json`, wire it explicitly in the Codex manifest with `"mcpServers": "./.mcp.json"`.

## Key Files

- `.claude-plugin/marketplace.json` — Claude plugin registry
- `.agents/plugins/marketplace.json` — Codex plugin registry
- `plugins/*/.claude-plugin/plugin.json` — Claude per-plugin metadata
- `plugins/*/.codex-plugin/plugin.json` — Codex per-plugin metadata
- `generate-summary.py` — auto-discovers plugins and generates mdBook SUMMARY.md
- `book.toml` — mdBook config (source: `book-src/`, output: `book-out/`)
- `.github/workflows/deploy-book.yml` — CI deploys to GitHub Pages on push to master

## Architecture Notes

- `book-src/` and `book-out/` are generated (gitignored) — never edit directly
- Build uses symlinks from `book-src/` to `plugins/` source files, so edits to plugin `.md` files are immediately visible during `serve`
- `generate-summary.py` extracts H1 headings from `.md` files for TOC titles, falling back to humanized filenames
- CI is triggered on pushes to `master` (not main)
