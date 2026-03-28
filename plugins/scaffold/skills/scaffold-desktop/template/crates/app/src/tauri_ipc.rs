#![allow(dead_code)]

use std::cell::RefCell;

use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

use crate::types::{ItemListResponse, ItemResponse};

// ── Raw JS bindings ──────────────────────────────────────────────

#[wasm_bindgen(inline_js = r#"
export async function tauri_invoke(cmd, args) {
    return await window.__TAURI__.core.invoke(cmd, args || {});
}

export async function tauri_listen(event, callback) {
    return await window.__TAURI__.event.listen(event, callback);
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

pub fn listen<T: DeserializeOwned + 'static>(
    event: &str,
    callback: impl FnMut(T) + 'static,
) {
    let event = event.to_string();
    let callback = RefCell::new(callback);
    wasm_bindgen_futures::spawn_local(async move {
        let closure = Closure::new(move |val: JsValue| {
            match serde_wasm_bindgen::from_value::<EventWrapper<T>>(val) {
                Ok(wrapper) => (callback.borrow_mut())(wrapper.payload),
                Err(e) => log::error!("Event parse error: {}", e),
            }
        });
        let _ = tauri_listen(&event, &closure).await;
        closure.forget();
    });
}

// ── Typed command wrappers ───────────────────────────────────────

pub async fn get_items() -> Result<ItemListResponse, String> {
    invoke("get_items", JsValue::NULL).await
}

pub async fn create_item(name: &str, quantity: u32) -> Result<ItemResponse, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "name": name,
        "quantity": quantity,
    }))
    .map_err(|e| e.to_string())?;
    invoke("create_item", args).await
}

pub async fn delete_item(id: &str) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "id": id,
    }))
    .map_err(|e| e.to_string())?;
    invoke_unit("delete_item", args).await
}
