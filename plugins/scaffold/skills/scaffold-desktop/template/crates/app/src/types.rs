use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemResponse {
    pub id: String,
    pub name: String,
    pub quantity: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemListResponse {
    pub items: Vec<ItemResponse>,
}
