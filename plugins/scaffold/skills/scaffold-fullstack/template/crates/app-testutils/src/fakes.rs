use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use app_core::domain::{Item, ItemId};
use app_core::error::ItemError;
use app_core::ports::ItemRepository;
use ras_auth_core::{AuthError, AuthFuture, AuthProvider, AuthenticatedUser};

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

/// Shared state backing a `FakeAuthProvider`. Clone the provider to share
/// the same user list across the test and the service builder.
#[derive(Clone)]
pub struct FakeAuthProvider {
    users: Arc<Mutex<Vec<(String, AuthenticatedUser)>>>,
}

impl FakeAuthProvider {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_user(&self, token: &str, user_id: &str, permissions: Vec<String>) {
        self.users.lock().unwrap().push((
            token.into(),
            AuthenticatedUser {
                user_id: user_id.into(),
                permissions: permissions.into_iter().collect::<HashSet<_>>(),
                metadata: None,
            },
        ));
    }
}

impl Default for FakeAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthProvider for FakeAuthProvider {
    fn authenticate(&self, token: String) -> AuthFuture<'_> {
        Box::pin(async move {
            self.users
                .lock()
                .unwrap()
                .iter()
                .find(|(t, _)| *t == token)
                .map(|(_, u)| u.clone())
                .ok_or(AuthError::InvalidToken)
        })
    }
}
