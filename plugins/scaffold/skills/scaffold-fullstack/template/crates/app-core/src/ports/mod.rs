use async_trait::async_trait;

use crate::domain::{Item, ItemId};
use crate::error::ItemError;

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn create(&self, item: &Item) -> Result<(), ItemError>;
    async fn get_by_id(&self, id: &ItemId) -> Result<Option<Item>, ItemError>;
    async fn list(&self) -> Result<Vec<Item>, ItemError>;
    async fn delete(&self, id: &ItemId) -> Result<(), ItemError>;
}
