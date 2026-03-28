use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use app_core::domain::{Item, ItemId};
use app_core::error::ItemError;
use app_core::ports::ItemRepository;

pub struct InMemoryItemRepository {
    items: Mutex<HashMap<ItemId, Item>>,
}

impl InMemoryItemRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryItemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ItemRepository for InMemoryItemRepository {
    async fn create(&self, item: &Item) -> Result<(), ItemError> {
        let mut items = self
            .items
            .lock()
            .map_err(|e| ItemError::Storage(e.to_string()))?;
        if items.contains_key(&item.id) {
            return Err(ItemError::AlreadyExists(item.id.clone()));
        }
        items.insert(item.id.clone(), item.clone());
        Ok(())
    }

    async fn get_by_id(&self, id: &ItemId) -> Result<Option<Item>, ItemError> {
        let items = self
            .items
            .lock()
            .map_err(|e| ItemError::Storage(e.to_string()))?;
        Ok(items.get(id).cloned())
    }

    async fn list(&self) -> Result<Vec<Item>, ItemError> {
        let items = self
            .items
            .lock()
            .map_err(|e| ItemError::Storage(e.to_string()))?;
        Ok(items.values().cloned().collect())
    }

    async fn delete(&self, id: &ItemId) -> Result<(), ItemError> {
        let mut items = self
            .items
            .lock()
            .map_err(|e| ItemError::Storage(e.to_string()))?;
        items
            .remove(id)
            .ok_or_else(|| ItemError::NotFound(id.clone()))?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_get_item() {
        let repo = InMemoryItemRepository::new();
        let item = Item::new("Widget".into(), 10);
        let id = item.id.clone();

        repo.create(&item).await.unwrap();
        let found = repo.get_by_id(&id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Widget");
    }

    #[tokio::test]
    async fn create_duplicate_fails() {
        let repo = InMemoryItemRepository::new();
        let item = Item::new("Widget".into(), 10);

        repo.create(&item).await.unwrap();
        let result = repo.create(&item).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_nonexistent_fails() {
        let repo = InMemoryItemRepository::new();
        let result = repo.delete(&ItemId::new()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_returns_all_items() {
        let repo = InMemoryItemRepository::new();
        repo.create(&Item::new("A".into(), 1)).await.unwrap();
        repo.create(&Item::new("B".into(), 2)).await.unwrap();

        let items = repo.list().await.unwrap();
        assert_eq!(items.len(), 2);
    }
}
