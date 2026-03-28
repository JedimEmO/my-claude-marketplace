use tauri::State;
use uuid::Uuid;

use app_core::domain::{Item, ItemId};
use app_core::dto::{ItemListResponse, ItemResponse};

use crate::state::AppState;

#[tauri::command]
pub async fn get_items(state: State<'_, AppState>) -> Result<ItemListResponse, String> {
    let items = state.items.list().await.map_err(|e| e.to_string())?;
    Ok(ItemListResponse {
        items: items.into_iter().map(ItemResponse::from).collect(),
    })
}

#[tauri::command]
pub async fn create_item(
    state: State<'_, AppState>,
    name: String,
    quantity: u32,
) -> Result<ItemResponse, String> {
    let item = Item::new(name, quantity);
    state.items.create(&item).await.map_err(|e| e.to_string())?;
    Ok(ItemResponse::from(item))
}

#[tauri::command]
pub async fn delete_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .items
        .delete(&ItemId(uuid))
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use app_core::domain::{Item, ItemId};
    use app_core::ports::ItemRepository;
    use app_testutils::{FakeItemRepository, an_item};

    #[tokio::test]
    async fn create_and_list_items() {
        let repo = Arc::new(FakeItemRepository::new());
        let item = Item::new("Widget".into(), 10);
        repo.create(&item).await.unwrap();

        let items = repo.list().await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Widget");
    }

    #[tokio::test]
    async fn delete_existing_item() {
        let repo = Arc::new(FakeItemRepository::new());
        let item = an_item().with_name("Gadget").build();
        let id = item.id.clone();
        repo.create(&item).await.unwrap();

        repo.delete(&id).await.unwrap();

        let items = repo.list().await.unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn delete_nonexistent_item_fails() {
        let repo = Arc::new(FakeItemRepository::new());
        let result = repo.delete(&ItemId::new()).await;
        assert!(result.is_err());
    }
}
