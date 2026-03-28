---
name: dwind-tauri
description: This skill should be used when the user asks to build a Tauri desktop application with a dwind/dominator frontend, set up Tauri with Rust WASM UI, create Tauri commands or IPC, handle Tauri events from dwind, configure tauri.conf.json, or asks about Tauri + dwind project structure.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Tauri + Dwind Desktop App

Build native desktop applications using Tauri 2 for the backend and dwind/dominator for the WASM frontend. The frontend compiles to WebAssembly and runs in Tauri's webview, communicating with a native Rust backend via IPC.

## Project Structure

```
my-app/
├── Cargo.toml              # Frontend (cdylib, WASM)
├── Trunk.toml              # WASM bundler config
├── public/
│   └── index.html          # HTML shell for Trunk
├── src/
│   ├── lib.rs              # WASM entry point (dwind + dominator)
│   ├── tauri_ipc.rs        # IPC bridge to Tauri backend
│   ├── state.rs            # Frontend reactive state (Mutable<T>)
│   └── components/         # UI components
└── src-tauri/              # Tauri backend (separate crate)
    ├── Cargo.toml
    ├── tauri.conf.json     # Tauri configuration
    ├── build.rs            # tauri_build::build()
    ├── capabilities/
    │   └── default.json    # Permission scoping
    └── src/
        ├── main.rs         # Tauri app builder
        ├── commands.rs     # IPC command handlers
        └── state.rs        # Backend state
```

**Critical**: The frontend and backend are separate crates with separate workspaces. The frontend compiles to `wasm32-unknown-unknown`; the backend compiles to the native target.

## Workspace Isolation

The frontend crate must be its own workspace (or excluded from the parent) because dwind path dependencies resolve against the dwind workspace. The backend joins the parent workspace normally.

```toml
# Parent workspace Cargo.toml
[workspace]
members = ["crates/my-app/src-tauri"]
exclude = ["crates/my-app"]  # Frontend excluded from parent workspace

# Frontend Cargo.toml
[workspace]
exclude = ["src-tauri"]  # Backend excluded from frontend workspace
```

## Frontend Setup

### Cargo.toml

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[workspace]
exclude = ["src-tauri"]

[dependencies]
dominator = "0.5"
dwind = "0.7"
dwind-macros = "0.7"
futures-signals = "0.3"
futures-signals-component-macro = { version = "0.4", features = ["dominator"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["Window", "console"] }
js-sys = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde-wasm-bindgen = "0.6"
log = "0.4"
wasm-log = "0.3"
```

### Trunk.toml

```toml
[build]
target = "public/index.html"

[watch]
ignore = ["./src-tauri"]

[serve]
port = 1420
ws_protocol = "ws"
```

### public/index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link data-trunk rel="rust" href="../Cargo.toml">
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

The `<link data-trunk rel="rust">` directive tells Trunk to compile the Rust crate to WASM.

### lib.rs — WASM Entry Point

```rust
#[macro_use]
extern crate dwind_macros;

use wasm_bindgen::prelude::*;
use std::rc::Rc;

mod tauri_ipc;
mod state;
mod components;

#[wasm_bindgen(start)]
pub async fn main() {
    wasm_log::init(wasm_log::Config::default());
    dwind::stylesheet();

    let state = Rc::new(state::AppState::new());

    // Wire up Tauri event listeners
    setup_event_listeners(state.clone());

    // Fetch initial data from backend
    {
        let state = state.clone();
        wasm_bindgen_futures::spawn_local(async move {
            // Call Tauri commands to populate initial state
            if let Ok(data) = tauri_ipc::get_initial_data().await {
                state.data.set(Some(data));
            }
        });
    }

    dominator::append_dom(&dominator::body(), components::app(state));
}

fn setup_event_listeners(state: Rc<state::AppState>) {
    tauri_ipc::listen::<String>("backend-event", move |payload| {
        // Update reactive state — UI updates automatically
        log::info!("Received: {}", payload);
    });
}
```

## Tauri IPC Bridge

The IPC bridge connects the dwind frontend to the Tauri backend. It uses `wasm_bindgen` inline JS to access `window.__TAURI__`.

### tauri_ipc.rs

```rust
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

// Raw JS bindings to Tauri global API
#[wasm_bindgen(inline_js = r#"
export async function tauri_invoke(cmd, args) {
    return await window.__TAURI__.core.invoke(cmd, args || {});
}

export async function tauri_listen(event, callback) {
    return await window.__TAURI__.event.listen(event, callback);
}

export function tauri_convert_file_src(path) {
    return window.__TAURI__.core.convertFileSrc(path);
}
"#)]
extern "C" {
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    async fn tauri_listen(event: &str, callback: &Closure<dyn Fn(JsValue)>) -> Result<JsValue, JsValue>;
    fn tauri_convert_file_src(path: &str) -> String;
}

// Generic typed invoke — serializes args, deserializes result
async fn invoke<T: DeserializeOwned>(cmd: &str, args: JsValue) -> Result<T, String> {
    let result = tauri_invoke(cmd, args)
        .await
        .map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

async fn invoke_unit(cmd: &str, args: JsValue) -> Result<(), String> {
    tauri_invoke(cmd, args)
        .await
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

// Event listener — deserializes Tauri event payload
#[derive(serde::Deserialize)]
struct EventWrapper<T> {
    payload: T,
}

pub fn listen<T: DeserializeOwned + 'static>(
    event: &str,
    mut callback: impl FnMut(T) + 'static,
) {
    let event = event.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let closure = Closure::new(move |val: JsValue| {
            match serde_wasm_bindgen::from_value::<EventWrapper<T>>(val) {
                Ok(wrapper) => callback(wrapper.payload),
                Err(e) => log::error!("Event parse error: {}", e),
            }
        });
        let _ = tauri_listen(&event, &closure).await;
        closure.forget(); // Must keep alive for app lifetime
    });
}

// Convert a filesystem path to an asset:// URL for the webview
pub fn convert_file_src(path: &str) -> String {
    tauri_convert_file_src(path)
}

// --- Typed command wrappers ---

pub async fn get_initial_data() -> Result<MyData, String> {
    invoke("get_initial_data", JsValue::NULL).await
}

pub async fn save_item(name: &str, value: &str) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "name": name,
        "value": value,
    })).map_err(|e| e.to_string())?;
    invoke_unit("save_item", args).await
}
```

**Important**: Tauri command argument names must be camelCase in the JSON (Tauri deserializes them that way), even though the Rust backend uses snake_case.

## Backend Setup

### src-tauri/Cargo.toml

```toml
[package]
name = "my-app-tauri"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2" }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Add features/plugins as needed:
- `tauri = { features = ["protocol-asset"] }` — serve files via `asset://` protocol
- `tauri-plugin-dialog = "2"` — native file/folder dialogs
- `tauri-plugin-shell = "2"` — open URLs in browser

### src-tauri/build.rs

```rust
fn main() {
    tauri_build::build();
}
```

### src-tauri/tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "My App",
  "version": "0.1.0",
  "identifier": "com.myapp.dev",
  "build": {
    "beforeDevCommand": "trunk serve --port 1420",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "trunk build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "My App",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "minWidth": 800,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.png"]
  }
}
```

**Key settings**:
- `withGlobalTauri: true` — injects `window.__TAURI__` so WASM can call it
- `beforeDevCommand` starts Trunk on port 1420
- `frontendDist: "../dist"` points to Trunk's output for production builds

### src-tauri/capabilities/default.json

```json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

Add permissions as needed: `"dialog:default"`, `"dialog:allow-open"`, `"shell:allow-open"`, etc.

### src-tauri/src/main.rs

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .manage(state::AppState {
            data: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_data,
            commands::save_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### src-tauri/src/commands.rs

```rust
use tauri::{AppHandle, State};
use crate::state::AppState;

#[tauri::command]
pub fn get_initial_data(state: State<AppState>) -> Result<MyData, String> {
    // Access managed state, return data
    Ok(MyData { /* ... */ })
}

#[tauri::command]
pub async fn save_item(
    app: AppHandle,
    name: String,
    value: String,
) -> Result<(), String> {
    // Do work, optionally emit events for progress
    app.emit("save-progress", 50).map_err(|e| e.to_string())?;
    Ok(())
}
```

**Command rules**:
- Return `Result<T, String>` for error handling
- Use `State<T>` to access managed state
- Use `AppHandle` for emitting events or accessing app resources
- Use `tauri::async_runtime::spawn_blocking()` for CPU-heavy work

## Key Patterns

### Pattern: Frontend calls backend command

```rust
// Frontend (WASM)
let result = tauri_ipc::save_item("key", "value").await;

// Backend (native)
#[tauri::command]
pub async fn save_item(name: String, value: String) -> Result<(), String> { ... }
```

### Pattern: Backend streams events to frontend

```rust
// Backend — emit during long operation
app.emit("processing-progress", ProgressPayload { percent: 50 })?;

// Frontend — listen and update reactive state
tauri_ipc::listen::<ProgressPayload>("processing-progress", move |p| {
    progress.set(p.percent);  // UI updates automatically
});
```

### Pattern: Serve files via asset protocol

```rust
// Backend — enable in tauri.conf.json: features = ["protocol-asset"]
// and security.assetProtocol.enable = true, scope = ["*/**"]

// Frontend — convert path to asset:// URL
let url = tauri_ipc::convert_file_src("/path/to/file.png");
// url = "asset://localhost/path/to/file.png"
// Use in img src, audio src, fetch(), etc.
```

### Pattern: State on both sides

```rust
// Backend: thread-safe with Mutex (accessed from multiple commands)
pub struct AppState {
    pub data: Mutex<Option<MyData>>,
}

// Frontend: reactive with Mutable (drives UI updates)
pub struct AppState {
    pub data: Mutable<Option<MyData>>,
    pub loading: Mutable<bool>,
}
```

## Development

```bash
# Prerequisites
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli  # or: cargo install create-tauri-app

# Development (starts Trunk + Tauri together)
cd src-tauri && cargo tauri dev

# Production build
cd src-tauri && cargo tauri build
```

`cargo tauri dev` automatically runs `beforeDevCommand` (Trunk) and opens the app window. Hot-reload works for frontend changes.

## Reference App

For a complete working example of Tauri + dwind/dominator with IPC, events, file access, and audio playback:
`/home/mmy/repos/ai/experiments/karaokemonster/crates/karaoke-app/`
