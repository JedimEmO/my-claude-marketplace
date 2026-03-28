use app_core::domain::{Item, ItemId};
use chrono::Utc;
use uuid::Uuid;

#[must_use]
pub struct ItemBuilder {
    id: Uuid,
    name: String,
    quantity: u32,
}

impl Default for ItemBuilder {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Test Item".into(),
            quantity: 100,
        }
    }
}

impl ItemBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn out_of_stock(mut self) -> Self {
        self.quantity = 0;
        self
    }

    pub fn build(self) -> Item {
        Item {
            id: ItemId(self.id),
            name: self.name,
            quantity: self.quantity,
            created_at: Utc::now(),
        }
    }
}

pub fn an_item() -> ItemBuilder {
    ItemBuilder::default()
}
