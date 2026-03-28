use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;

use crate::types::ItemResponse;

pub struct AppState {
    pub items: MutableVec<ItemResponse>,
    pub loading: Mutable<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            items: MutableVec::new(),
            loading: Mutable::new(false),
        }
    }
}
