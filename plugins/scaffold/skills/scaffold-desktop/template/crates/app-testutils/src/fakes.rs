use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use app_core::domain::{Item, ItemId};
use app_core::error::ItemError;
use app_core::ports::ItemRepository;

pub struct FakeItemRepository {
    items: Mutex<HashMap<ItemId, Item>>,
    should_fail: Mutex<bool>,
}

impl FakeItemRepository {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
            should_fail: Mutex::new(false),
        }
    }

    pub fn with_items(items: Vec<Item>) -> Self {
        let map = items.into_iter().map(|i| (i.id.clone(), i)).collect();
        Self {
            items: Mutex::new(map),
            should_fail: Mutex::new(false),
        }
    }

    pub fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    pub fn get_item(&self, id: &ItemId) -> Option<Item> {
        self.items.lock().unwrap().get(id).cloned()
    }

    pub fn item_count(&self) -> usize {
        self.items.lock().unwrap().len()
    }

    fn check_failure(&self) -> Result<(), ItemError> {
        if *self.should_fail.lock().unwrap() {
            return Err(ItemError::Storage("fake failure".into()));
        }
        Ok(())
    }
}

impl Default for FakeItemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ItemRepository for FakeItemRepository {
    async fn create(&self, item: &Item) -> Result<(), ItemError> {
        self.check_failure()?;
        let mut items = self.items.lock().unwrap();
        if items.contains_key(&item.id) {
            return Err(ItemError::AlreadyExists(item.id.clone()));
        }
        items.insert(item.id.clone(), item.clone());
        Ok(())
    }

    async fn get_by_id(&self, id: &ItemId) -> Result<Option<Item>, ItemError> {
        self.check_failure()?;
        Ok(self.items.lock().unwrap().get(id).cloned())
    }

    async fn list(&self) -> Result<Vec<Item>, ItemError> {
        self.check_failure()?;
        Ok(self.items.lock().unwrap().values().cloned().collect())
    }

    async fn delete(&self, id: &ItemId) -> Result<(), ItemError> {
        self.check_failure()?;
        self.items
            .lock()
            .unwrap()
            .remove(id)
            .ok_or_else(|| ItemError::NotFound(id.clone()))?;
        Ok(())
    }
}
