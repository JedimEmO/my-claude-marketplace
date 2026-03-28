use ras_rest_macro::rest_service;

use crate::types::{CreateItemRequest, ItemListResponse, ItemResponse};

rest_service!({
    service_name: ItemService,
    base_path: "/api/v1",
    openapi: true,
    serve_docs: true,
    docs_path: "/docs",
    endpoints: [
        GET UNAUTHORIZED items() -> ItemListResponse,
        GET UNAUTHORIZED items/{id: String}() -> ItemResponse,
        POST WITH_PERMISSIONS(["items:write"]) items(CreateItemRequest) -> ItemResponse,
        DELETE WITH_PERMISSIONS(["items:write"]) items/{id: String}() -> (),
    ]
});
