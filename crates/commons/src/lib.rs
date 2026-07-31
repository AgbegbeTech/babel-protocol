use chrono::{DateTime, Utc};
use consent::PublicationScope;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommonsPublication {
    pub id: Uuid,
    pub artifact_id: Uuid,
    pub title: String,
    pub summary: String,
    pub revision_hash: String,
    pub language_tags: Vec<String>,
    pub cultural_context_tags: Vec<String>,
    pub evidence_tags: Vec<String>,
    pub publication_scope: PublicationScope,
    pub consent_receipt_ids: Vec<Uuid>,
    pub ai_disclosure: String,
    pub published_at: DateTime<Utc>,
}

impl CommonsPublication {
    pub fn transcript_exposed(&self) -> bool {
        false
    }
}
