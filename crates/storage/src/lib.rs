use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectReference {
    pub id: String,
    pub provider: String,
    pub bucket: String,
    pub key: String,
    pub content_hash: String,
    pub privacy_classification: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("object store unavailable")]
    Unavailable,
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put(
        &self,
        key: &str,
        bytes: &[u8],
        privacy_classification: &str,
    ) -> Result<ObjectReference, StorageError>;
}
