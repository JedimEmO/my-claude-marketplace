use std::sync::Arc;

use app_core::ports::ItemRepository;

pub struct AppState {
    pub items: Arc<dyn ItemRepository>,
}
