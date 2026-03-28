use std::sync::Arc;

use async_trait::async_trait;
use ras_auth_core::AuthenticatedUser;
use ras_rest_core::{RestError, RestResponse, RestResult};
use uuid::Uuid;

use app_api::{CreateItemRequest, ItemListResponse, ItemResponse, ItemServiceTrait};
use app_core::domain::Item;
use app_core::error::ItemError;
use app_core::ports::ItemRepository;

pub struct ItemServiceHandler {
    items: Arc<dyn ItemRepository>,
}

impl ItemServiceHandler {
    pub fn new(items: Arc<dyn ItemRepository>) -> Self {
        Self { items }
    }
}

fn item_error_to_rest(e: ItemError) -> RestError {
    match e {
        ItemError::NotFound(id) => RestError::not_found(format!("Item not found: {id}")),
        ItemError::AlreadyExists(id) => {
            RestError::bad_request(format!("Item already exists: {id}"))
        }
        ItemError::Storage(msg) => RestError::internal_server_error(msg),
    }
}

#[async_trait]
impl ItemServiceTrait for ItemServiceHandler {
    async fn get_items(&self) -> RestResult<ItemListResponse> {
        let items = self.items.list().await.map_err(item_error_to_rest)?;
        let response = ItemListResponse {
            items: items.into_iter().map(ItemResponse::from).collect(),
        };
        Ok(RestResponse::ok(response))
    }

    async fn get_items_by_id(&self, id: String) -> RestResult<ItemResponse> {
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| RestError::bad_request(format!("Invalid UUID: {id}")))?;

        let item = self
            .items
            .get_by_id(&app_core::domain::ItemId(uuid))
            .await
            .map_err(item_error_to_rest)?
            .ok_or_else(|| RestError::not_found(format!("Item not found: {id}")))?;

        Ok(RestResponse::ok(ItemResponse::from(item)))
    }

    async fn post_items(
        &self,
        _user: &AuthenticatedUser,
        req: CreateItemRequest,
    ) -> RestResult<ItemResponse> {
        let item = Item::new(req.name, req.quantity);
        self.items.create(&item).await.map_err(item_error_to_rest)?;
        Ok(RestResponse::created(ItemResponse::from(item)))
    }

    async fn delete_items_by_id(&self, _user: &AuthenticatedUser, id: String) -> RestResult<()> {
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| RestError::bad_request(format!("Invalid UUID: {id}")))?;

        self.items
            .delete(&app_core::domain::ItemId(uuid))
            .await
            .map_err(item_error_to_rest)?;

        Ok(RestResponse::ok(()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use app_testutils::{FakeAuthProvider, FakeItemRepository, an_item};
    use axum_test::TestServer;

    use super::*;

    struct TestApp {
        server: TestServer,
        auth: FakeAuthProvider,
        items: Arc<FakeItemRepository>,
    }

    impl TestApp {
        fn new() -> Self {
            let items = Arc::new(FakeItemRepository::new());
            let auth = FakeAuthProvider::new();

            let handler = ItemServiceHandler::new(Arc::clone(&items) as Arc<dyn ItemRepository>);

            let router = app_api::ItemServiceBuilder::new(handler)
                .auth_provider(auth.clone())
                .build();

            let server = TestServer::new(router).unwrap();
            Self {
                server,
                auth,
                items,
            }
        }
    }

    #[tokio::test]
    async fn list_items_returns_empty_when_no_items() {
        let app = TestApp::new();

        let response = app.server.get("/api/v1/items").await;
        response.assert_status_ok();

        let body: ItemListResponse = response.json();
        assert!(body.items.is_empty());
    }

    #[tokio::test]
    async fn list_items_returns_existing_items() {
        let app = TestApp::new();
        let item = an_item().with_name("Widget").build();
        app.items.create(&item).await.unwrap();

        let response = app.server.get("/api/v1/items").await;
        response.assert_status_ok();

        let body: ItemListResponse = response.json();
        assert_eq!(body.items.len(), 1);
        assert_eq!(body.items[0].name, "Widget");
    }

    #[tokio::test]
    async fn create_item_requires_auth() {
        let app = TestApp::new();

        let response = app
            .server
            .post("/api/v1/items")
            .json(&serde_json::json!({ "name": "Widget", "quantity": 10 }))
            .await;

        response.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn create_item_with_valid_token() {
        let app = TestApp::new();
        app.auth
            .add_user("test-token", "alice", vec!["items:write".into()]);

        let response = app
            .server
            .post("/api/v1/items")
            .add_header(
                axum::http::header::AUTHORIZATION,
                axum::http::HeaderValue::from_static("Bearer test-token"),
            )
            .json(&serde_json::json!({ "name": "Widget", "quantity": 10 }))
            .await;

        response.assert_status(axum::http::StatusCode::CREATED);

        let body: ItemResponse = response.json();
        assert_eq!(body.name, "Widget");
        assert_eq!(body.quantity, 10);
    }

    #[tokio::test]
    async fn get_item_by_id() {
        let app = TestApp::new();
        let item = an_item().with_name("Gadget").build();
        let id = item.id.0.to_string();
        app.items.create(&item).await.unwrap();

        let response = app.server.get(&format!("/api/v1/items/{id}")).await;
        response.assert_status_ok();

        let body: ItemResponse = response.json();
        assert_eq!(body.name, "Gadget");
    }

    #[tokio::test]
    async fn get_nonexistent_item_returns_404() {
        let app = TestApp::new();
        let fake_id = Uuid::new_v4();

        let response = app.server.get(&format!("/api/v1/items/{fake_id}")).await;

        response.assert_status_not_found();
    }
}
