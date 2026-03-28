# Tauri IPC Bridge Template

Complete, copy-pasteable `tauri_ipc.rs` module for a dwind/dominator frontend.

## Full Module

```rust
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

// ── Raw JS bindings ──────────────────────────────────────────────

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

export async function tauri_dialog_open(options) {
    return await window.__TAURI__.dialog.open(options || {});
}

export async function tauri_dialog_save(options) {
    return await window.__TAURI__.dialog.save(options || {});
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn tauri_listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> Result<JsValue, JsValue>;

    fn tauri_convert_file_src(path: &str) -> String;

    #[wasm_bindgen(catch)]
    async fn tauri_dialog_open(options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn tauri_dialog_save(options: JsValue) -> Result<JsValue, JsValue>;
}

// ── Generic helpers ──────────────────────────────────────────────

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

// ── Event listener ───────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct EventWrapper<T> {
    payload: T,
}

/// Listen for Tauri events emitted by the backend.
/// The callback receives the deserialized payload.
/// The listener lives for the lifetime of the app.
pub fn listen<T: DeserializeOwned + 'static>(
    event: &str,
    mut callback: impl FnMut(T) + 'static,
) {
    let event = event.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let closure = Closure::new(move |val: JsValue| {
            match serde_wasm_bindgen::from_value::<EventWrapper<T>>(val) {
                Ok(wrapper) => callback(wrapper.payload),
                Err(e) => log::error!("Failed to parse event '{}': {}", "<event>", e),
            }
        });
        let _ = tauri_listen(&event, &closure).await;
        closure.forget();
    });
}

// ── Asset protocol ───────────────────────────────────────────────

/// Convert an absolute file path to an asset:// URL loadable by the webview.
/// Requires `protocol-asset` feature and assetProtocol enabled in tauri.conf.json.
pub fn convert_file_src(path: &str) -> String {
    tauri_convert_file_src(path)
}

// ── File dialogs (requires tauri-plugin-dialog) ──────────────────

/// Open a native file picker. Returns the selected file path, or None if cancelled.
pub async fn pick_file(title: &str, filters: &[(&str, &[&str])]) -> Result<Option<String>, String> {
    let filter_array: Vec<serde_json::Value> = filters
        .iter()
        .map(|(name, exts)| {
            serde_json::json!({
                "name": name,
                "extensions": exts,
            })
        })
        .collect();

    let options = serde_wasm_bindgen::to_value(&serde_json::json!({
        "title": title,
        "filters": filter_array,
    }))
    .map_err(|e| e.to_string())?;

    let result = tauri_dialog_open(options)
        .await
        .map_err(|e| format!("{:?}", e))?;

    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }
    Ok(result.as_string())
}

/// Open a native directory picker. Returns the selected path, or None if cancelled.
pub async fn pick_directory(title: &str) -> Result<Option<String>, String> {
    let options = serde_wasm_bindgen::to_value(&serde_json::json!({
        "title": title,
        "directory": true,
    }))
    .map_err(|e| e.to_string())?;

    let result = tauri_dialog_open(options)
        .await
        .map_err(|e| format!("{:?}", e))?;

    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }
    Ok(result.as_string())
}

// ── App commands (add your typed wrappers below) ─────────────────

// Example:
//
// #[derive(serde::Deserialize)]
// pub struct MyData {
//     pub name: String,
//     pub count: u32,
// }
//
// pub async fn get_data() -> Result<MyData, String> {
//     invoke("get_data", JsValue::NULL).await
// }
//
// pub async fn save_data(name: &str, count: u32) -> Result<(), String> {
//     let args = serde_wasm_bindgen::to_value(&serde_json::json!({
//         "name": name,    // camelCase keys for Tauri
//         "count": count,
//     })).map_err(|e| e.to_string())?;
//     invoke_unit("save_data", args).await
// }
```

## Usage Notes

- **camelCase args**: Tauri deserializes command arguments as camelCase JSON keys, so use `"projectName"` not `"project_name"` in the `json!()` macro, even though the backend Rust function uses `project_name: String`.
- **`Closure::forget()`**: Event listeners must call `.forget()` to prevent the closure from being dropped. This leaks memory intentionally — listeners live for the app's lifetime.
- **Dialog plugin**: `pick_file` and `pick_directory` require `tauri-plugin-dialog` in the backend and `"dialog:default"` + `"dialog:allow-open"` in capabilities.
- **Asset protocol**: `convert_file_src` requires `features = ["protocol-asset"]` on the `tauri` dependency and `security.assetProtocol.enable = true` in `tauri.conf.json`.
