pub mod components;

#[cfg(target_arch = "wasm32")]
mod entry {
    use wasm_bindgen::prelude::*;

    use super::components::app::app_root;

    #[wasm_bindgen(start)]
    pub async fn main() {
        console_error_panic_hook::set_once();
        dwind::stylesheet();
        dominator::append_dom(&dominator::body(), app_root());
    }
}
