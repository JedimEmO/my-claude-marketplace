// API types re-exported from app-core with JsonSchema added for OpenAPI generation.
// The macro-generated code requires JsonSchema on all request/response types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateItemRequest {
    pub name: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ItemResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub quantity: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ItemListResponse {
    pub items: Vec<ItemResponse>,
}

impl From<app_core::domain::Item> for ItemResponse {
    fn from(item: app_core::domain::Item) -> Self {
        Self {
            id: item.id.0,
            name: item.name,
            quantity: item.quantity,
            created_at: item.created_at,
        }
    }
}
