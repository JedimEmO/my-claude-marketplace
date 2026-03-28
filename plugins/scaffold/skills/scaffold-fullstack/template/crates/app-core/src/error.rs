use crate::domain::ItemId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ItemError {
    #[error("item not found: {0}")]
    NotFound(ItemId),
    #[error("item already exists: {0}")]
    AlreadyExists(ItemId),
    #[error("storage error: {0}")]
    Storage(String),
}
