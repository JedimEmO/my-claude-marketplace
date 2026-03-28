---
name: dwind-project-setup
description: This skill should be used when the user asks to create a new dwind project, set up dwind in an existing project, configure the Rust-to-WASM build pipeline, or asks about dwind project structure, Cargo.toml dependencies, rollup config, or wasm-pack setup.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Dwind Project Setup — Scaffolding & Build Config

Set up a new Rust/WASM web application using the dwind stack.

## Project Structure

```
my-app/
├── Cargo.toml              # Rust dependencies
├── package.json            # npm: rollup, wasm-pack tools
├── rollup.config.js        # Build: Rust → WASM → JS bundle
├── index.html              # Minimal HTML shell
└── src/
    ├── lib.rs              # Entry point
    └── components/
        └── mod.rs          # Component modules
```

## Key Files

### Cargo.toml

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
dominator = "0.5"
dwind = "0.7"
dwind-macros = "0.7"
futures-signals = "0.3"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
console_error_panic_hook = "0.1"
```

Add the component library as needed:
```toml
dwui = { git = "https://github.com/user/dwind.git" }
```

### lib.rs — Entry Point

```rust
#[macro_use]
extern crate dwind_macros;  // Required for dwclass! / dwclass_signal!

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() {
    console_error_panic_hook::set_once();
    dwind::stylesheet();  // Initialize base utility stylesheets
    dominator::append_dom(&dominator::body(), app());
}

fn app() -> Dom {
    html!("div", {
        .dwclass!("min-h-screen bg-gray-950 text-white p-8")
        .text("Hello, dwind!")
    })
}
```

**Critical**: `#[macro_use] extern crate dwind_macros` must be at the crate root. Without it, `dwclass!` is not available.

### index.html

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>My App</title>
    <style>
        html, body {
            margin: 0; padding: 0; min-height: 100vh;
            background: linear-gradient(135deg, #080614 0%, #1a1540 50%, #12101e 100%);
            background-attachment: fixed;
        }
    </style>
</head>
<body></body>
</html>
```

The background gradient makes glass/transparency effects visible.

### package.json

```json
{
  "private": true,
  "type": "module",
  "name": "my-app",
  "version": "0.1.0",
  "scripts": {
    "build": "rimraf dist/js && rollup --config",
    "build:release": "rimraf dist/js && RELEASE=true rollup --config",
    "start": "[ ! -f Cargo.lock ] && cargo check --target wasm32-unknown-unknown; rimraf dist/js && rollup --config --watch"
  },
  "devDependencies": {
    "@rollup/plugin-terser": "^0",
    "@wasm-tool/rollup-plugin-rust": "^3",
    "binaryen": "^121",
    "rimraf": "^6",
    "rollup": "^4",
    "rollup-plugin-copy": "^3",
    "rollup-plugin-dev": "^2",
    "rollup-plugin-livereload": "^2",
    "rollup-plugin-terser": "^7"
  }
}
```

For the exact, up-to-date package.json, read the template at:
`/home/mmy/repos/oss/dwind-dominator-template/package.json`

### rollup.config.js

For the exact, up-to-date rollup config, read the template at:
`/home/mmy/repos/oss/dwind-dominator-template/rollup.config.js`

The key setup: `@wasm-tool/rollup-plugin-rust` compiles the Rust crate to WASM automatically. No manual `wasm-pack` commands needed. Dev builds include debug symbols; release uses `-Oz` + wasm-opt.

## Design System Crate (Optional)

For reusable component libraries, create a separate workspace crate:

```
crates/my-design-system/
├── Cargo.toml
├── build.rs                    # CSS codegen
├── resources/css/
│   └── tokens.css              # Utility classes referencing CSS vars
└── src/
    ├── lib.rs
    ├── theme/mod.rs            # Theme struct + CSS variable generation
    └── components/
        └── mod.rs
```

### build.rs for CSS Codegen

```rust
use dominator_css_bindgen::css::generate_rust_bindings_from_file;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let css_dir = PathBuf::from("resources/css");

    generate_rust_bindings_from_file(
        &css_dir.join("tokens.css"),
        &out_dir.join("tokens.rs"),
    );

    println!("cargo:rerun-if-changed=resources/css/");
}
```

Include the generated module:

```rust
pub mod tokens_css {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}
```

## Build Commands

```bash
# Prerequisites
rustup target add wasm32-unknown-unknown
npm install

# Development (hot-reload on port 8080)
npm start

# Production build
npm run build:release
```

## Tauri Desktop App

Dwind apps can also run as native desktop applications using Tauri 2. Instead of Rollup, the frontend is built with Trunk and loaded into Tauri's webview. The backend is a separate Rust crate that communicates with the frontend via IPC commands and events.

For Tauri setup, use the **dwind-tauri** skill which covers the full project structure, IPC bridge, Tauri configuration, and build toolchain.

Key differences from a web app:
- **Trunk** replaces Rollup as the WASM bundler
- Frontend and backend are **separate crates** with isolated workspaces
- `tauri.conf.json` wires Trunk's dev server to Tauri's webview
- `window.__TAURI__` provides IPC, accessed via `wasm_bindgen` inline JS

## Template Reference

For the most up-to-date, working project template with all configuration files:
- **Web app**: `/home/mmy/repos/oss/dwind-dominator-template/`
- **Tauri app**: `/home/mmy/repos/ai/experiments/karaokemonster/crates/karaoke-app/`

Read those files when scaffolding a new project to ensure you have the latest dependency versions and build config.
