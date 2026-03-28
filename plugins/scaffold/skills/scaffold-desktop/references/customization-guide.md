# Customization Guide

## Renaming the Project

Replace all `app-` prefixes with your project name. For a project named `acme`:

1. Rename directories: `app-core` → `acme-core`, `app-adapters` → `acme-adapters`, etc.
2. Update `Cargo.toml` names and path references in all crates
3. Update `use` statements: `app_core` → `acme_core`, etc.
4. Update workspace `Cargo.toml` members and exclude paths
5. Update `tauri.conf.json`: `productName`, `identifier`, window `title`
6. Rename the frontend crate (`app` → `acme`) in `crates/app/Cargo.toml`

## Replacing the Domain

The scaffold uses an Item/Inventory domain. To replace it:

1. **`app-core/src/domain/mod.rs`** — Replace `Item`, `ItemId` with your domain types
2. **`app-core/src/error.rs`** — Replace `ItemError` variants with your domain errors
3. **`app-core/src/ports/mod.rs`** — Replace `ItemRepository` with your domain trait(s)
4. **`app-core/src/dto.rs`** — Replace request/response types
5. **`app-adapters/src/in_memory.rs`** — Implement the new trait (or replace with a real adapter)
6. **`app-testutils/src/fakes.rs`** — Write fakes for your new traits
7. **`app-testutils/src/builders.rs`** — Write builders for your new domain types
8. **`app-tauri/src/commands.rs`** — Rewrite Tauri commands for the new domain
9. **`app-tauri/src/state.rs`** — Update `AppState` with the new trait(s)
10. **`src/types.rs`** (frontend) — Mirror the new DTOs with String-based fields
11. **`src/tauri_ipc.rs`** (frontend) — Update typed command wrappers
12. **`src/components/items.rs`** (frontend) — Rewrite the UI for the new domain

## Adding a Real Database

Replace `InMemoryItemRepository` with a Diesel + SQLite adapter:

1. Add `diesel` and `diesel_migrations` to workspace deps
2. Create a `SqliteItemRepository` in `app-adapters` (see **rust-architecture** skill's trait-patterns reference)
3. Wrap `SqliteConnection` in `Mutex` for `Send + Sync`
4. Update `app-tauri/src/main.rs` to create the connection and wire the new adapter
5. Store the database file in the Tauri app data directory (use `app.path().app_data_dir()`)

## Adding Tauri Plugins

### File Dialogs

1. Add `tauri-plugin-dialog = "2"` to `app-tauri/Cargo.toml`
2. Register: `.plugin(tauri_plugin_dialog::init())` in `app-tauri/src/main.rs`
3. Add `"dialog:default"`, `"dialog:allow-open"` to `app-tauri/capabilities/default.json`
4. Use the full IPC template from the **dwind-tauri** skill's reference which includes `pick_file` and `pick_directory`

### Shell (open URLs in browser)

1. Add `tauri-plugin-shell = "2"` to `app-tauri/Cargo.toml`
2. Register: `.plugin(tauri_plugin_shell::init())` in `app-tauri/src/main.rs`
3. Add `"shell:allow-open"` to `app-tauri/capabilities/default.json`

### File System Access

1. Add `tauri-plugin-fs = "2"` to `app-tauri/Cargo.toml`
2. Register: `.plugin(tauri_plugin_fs::init())` in `app-tauri/src/main.rs`
3. Add appropriate `"fs:*"` permissions to `app-tauri/capabilities/default.json`

### Asset Protocol (serve local files in webview)

1. Add `features = ["protocol-asset"]` to the `tauri` dependency
2. Enable in `tauri.conf.json`: `"security": { "assetProtocol": { "enable": true, "scope": ["**"] } }`
3. Use `tauri_ipc::convert_file_src(path)` to convert paths to `asset://` URLs

## Adding Backend Events

The backend can stream events to the frontend:

```rust
// Backend (commands.rs)
#[tauri::command]
pub async fn long_operation(app: tauri::AppHandle) -> Result<(), String> {
    app.emit("progress", 50).map_err(|e| e.to_string())?;
    // ...
    app.emit("progress", 100).map_err(|e| e.to_string())?;
    Ok(())
}
```

```rust
// Frontend (lib.rs)
tauri_ipc::listen::<u32>("progress", move |percent| {
    progress_state.set(percent);
});
```

## Adding More Tauri Commands

1. Add the `#[tauri::command]` function in `app-tauri/src/commands.rs`
2. Register it in `generate_handler![...]` in `app-tauri/src/main.rs`
3. Add a typed wrapper in `src/tauri_ipc.rs` using `invoke` or `invoke_unit`
4. Call from the frontend component
