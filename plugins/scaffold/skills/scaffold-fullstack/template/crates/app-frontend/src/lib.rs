use wasm_bindgen::prelude::*;

pub mod components;

use components::app::app_root;

#[wasm_bindgen(start)]
pub async fn main() {
    console_error_panic_hook::set_once();
    dwind::stylesheet();
    dominator::append_dom(&dominator::body(), app_root());
}
