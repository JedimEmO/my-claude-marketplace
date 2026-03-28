#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use std::sync::Arc;

use app_adapters::in_memory::InMemoryItemRepository;
use app_core::ports::ItemRepository;

fn main() {
    let items: Arc<dyn ItemRepository> = Arc::new(InMemoryItemRepository::new());

    tauri::Builder::default()
        .manage(state::AppState { items })
        .invoke_handler(tauri::generate_handler![
            commands::get_items,
            commands::create_item,
            commands::delete_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
