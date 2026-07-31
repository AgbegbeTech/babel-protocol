use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditRecord {
    pub id: Uuid,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub privacy_classification: String,
    pub content_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditRecord {
    pub fn new_without_body(
        actor_id: impl Into<String>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            actor_id: actor_id.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            privacy_classification: "metadata_only".to_string(),
            content_hash: None,
            created_at: Utc::now(),
        }
    }
}
