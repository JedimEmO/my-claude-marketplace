# Fake Examples — Supplementary Patterns

Examples that complement the core patterns in the SKILL.md. These cover additional scenarios: configurable failure, notification fakes, inventory domain, and testutils crate layout.

## Fake with Configurable Failure Modes

```rust
use std::sync::Mutex;

pub struct FakeNotifier {
    notifications: Mutex<Vec<(UserId, String)>>,
    fail_on: Mutex<Option<String>>,
}

impl FakeNotifier {
    pub fn new() -> Self {
        Self {
            notifications: Mutex::new(Vec::new()),
            fail_on: Mutex::new(None),
        }
    }

    pub fn fail_on(&self, notification_type: &str) {
        *self.fail_on.lock().unwrap() = Some(notification_type.into());
    }

    pub fn was_notified(&self, user_id: &UserId, notification_type: &str) -> bool {
        self.notifications.lock().unwrap().iter()
            .any(|(id, t)| id == user_id && t == notification_type)
    }

    pub fn all_notifications(&self) -> Vec<(UserId, String)> {
        self.notifications.lock().unwrap().clone()
    }
}

impl Notifier for FakeNotifier {
    fn send_welcome(&self, user: &User) -> Result<(), NotifyError> {
        if self.fail_on.lock().unwrap().as_deref() == Some("welcome") {
            return Err(NotifyError::SendFailed("fake failure".into()));
        }
        self.notifications.lock().unwrap().push((user.id.clone(), "welcome".into()));
        Ok(())
    }

    fn send_password_reset(&self, user: &User, _token: &str) -> Result<(), NotifyError> {
        if self.fail_on.lock().unwrap().as_deref() == Some("password_reset") {
            return Err(NotifyError::SendFailed("fake failure".into()));
        }
        self.notifications.lock().unwrap().push((user.id.clone(), "password_reset".into()));
        Ok(())
    }
}
```

## Second Domain Example — Inventory

A different domain to show the pattern generalizes. Same structure: trait in core, fake in testutils.

### Trait (core crate)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ItemId(pub String);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub quantity: u32,
}

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("item not found: {0:?}")]
    NotFound(ItemId),
    #[error("insufficient stock: have {available}, need {requested}")]
    InsufficientStock { available: u32, requested: u32 },
    #[error("storage error: {0}")]
    Storage(String),
}

pub trait InventoryRepository: Send + Sync {
    fn get(&self, id: &ItemId) -> Result<Option<Item>, InventoryError>;
    fn save(&self, item: &Item) -> Result<(), InventoryError>;
    fn reserve(&self, id: &ItemId, quantity: u32) -> Result<Item, InventoryError>;
}
```

### Fake (testutils crate)

```rust
use std::sync::Mutex;

pub struct FakeInventoryRepository {
    items: Mutex<Vec<Item>>,
    should_fail: Mutex<bool>,
}

impl FakeInventoryRepository {
    pub fn new() -> Self {
        Self { items: Mutex::new(Vec::new()), should_fail: Mutex::new(false) }
    }

    pub fn with_items(items: Vec<Item>) -> Self {
        Self { items: Mutex::new(items), should_fail: Mutex::new(false) }
    }

    pub fn set_should_fail(&self, fail: bool) { *self.should_fail.lock().unwrap() = fail; }
    pub fn get_item(&self, id: &ItemId) -> Option<Item> {
        self.items.lock().unwrap().iter().find(|i| i.id == *id).cloned()
    }
}

impl InventoryRepository for FakeInventoryRepository {
    fn get(&self, id: &ItemId) -> Result<Option<Item>, InventoryError> {
        if *self.should_fail.lock().unwrap() {
            return Err(InventoryError::Storage("fake failure".into()));
        }
        Ok(self.items.lock().unwrap().iter().find(|i| i.id == *id).cloned())
    }

    fn save(&self, item: &Item) -> Result<(), InventoryError> {
        if *self.should_fail.lock().unwrap() {
            return Err(InventoryError::Storage("fake failure".into()));
        }
        let mut items = self.items.lock().unwrap();
        if let Some(pos) = items.iter().position(|i| i.id == item.id) {
            items[pos] = item.clone();
        } else {
            items.push(item.clone());
        }
        Ok(())
    }

    fn reserve(&self, id: &ItemId, quantity: u32) -> Result<Item, InventoryError> {
        if *self.should_fail.lock().unwrap() {
            return Err(InventoryError::Storage("fake failure".into()));
        }
        let mut items = self.items.lock().unwrap();
        let item = items.iter_mut()
            .find(|i| i.id == *id)
            .ok_or_else(|| InventoryError::NotFound(id.clone()))?;

        if item.quantity < quantity {
            return Err(InventoryError::InsufficientStock {
                available: item.quantity,
                requested: quantity,
            });
        }

        item.quantity -= quantity;
        Ok(item.clone())
    }
}
```

## Item Builder

```rust
pub struct ItemBuilder {
    id: String,
    name: String,
    quantity: u32,
}

impl Default for ItemBuilder {
    fn default() -> Self {
        Self { id: "item-1".into(), name: "Test Item".into(), quantity: 100 }
    }
}

impl ItemBuilder {
    pub fn with_id(mut self, id: impl Into<String>) -> Self { self.id = id.into(); self }
    pub fn with_name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn with_quantity(mut self, quantity: u32) -> Self { self.quantity = quantity; self }
    pub fn out_of_stock(mut self) -> Self { self.quantity = 0; self }

    pub fn build(self) -> Item {
        Item { id: ItemId(self.id), name: self.name, quantity: self.quantity }
    }
}

pub fn an_item() -> ItemBuilder { ItemBuilder::default() }
```

## Testutils Crate Layout

```
crates/my-testutils/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── fakes/
    │   ├── mod.rs          # pub use each fake
    │   ├── user.rs          # FakeUserRepository
    │   ├── inventory.rs     # FakeInventoryRepository
    │   └── notifier.rs      # FakeNotifier
    └── builders/
        ├── mod.rs           # pub use each builder
        ├── user.rs          # UserBuilder, a_user()
        └── item.rs          # ItemBuilder, an_item()
```

```toml
# crates/my-testutils/Cargo.toml
[package]
name = "my-testutils"
version = "0.1.0"
edition.workspace = true
publish = false

[lints]
workspace = true

[dependencies]
my-core = { path = "../my-core" }
```

## Example Integration Test

```rust
use my_testutils::{FakeInventoryRepository, FakeNotifier, an_item, a_user};
use std::sync::Arc;

#[test]
fn placing_order_reserves_stock_and_notifies_user() {
    let inventory = Arc::new(FakeInventoryRepository::with_items(vec![
        an_item().with_id("widget").with_quantity(10).build()
    ]));
    let users = Arc::new(FakeUserRepository::with_users(vec![
        a_user().with_id("alice").build()
    ]));
    let notifier = Arc::new(FakeNotifier::new());

    let service = OrderService::new(
        Arc::clone(&inventory) as Arc<dyn InventoryRepository>,
        Arc::clone(&users) as Arc<dyn UserRepository>,
        Arc::clone(&notifier) as Arc<dyn Notifier>,
    );

    let order = service.place_order("alice", "widget", 3).unwrap();

    assert_eq!(order.quantity, 3);
    assert_eq!(inventory.get_item(&ItemId("widget".into())).unwrap().quantity, 7);
    assert!(notifier.was_notified(&UserId("alice".into()), "order_confirmation"));
}
```
