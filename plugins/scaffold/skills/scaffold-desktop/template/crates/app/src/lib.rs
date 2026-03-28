use std::rc::Rc;
use wasm_bindgen::prelude::*;

mod tauri_ipc;
mod types;
pub mod components;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn main() {
    wasm_log::init(wasm_log::Config::default());
    dwind::stylesheet();

    let state = Rc::new(components::state::AppState::new());

    // Load initial items from Tauri backend
    {
        let state = state.clone();
        wasm_bindgen_futures::spawn_local(async move {
            state.loading.set(true);
            if let Ok(response) = tauri_ipc::get_items().await {
                state.items.lock_mut().replace_cloned(response.items);
            }
            state.loading.set(false);
        });
    }

    dominator::append_dom(&dominator::body(), components::app::app_root(state));
}
