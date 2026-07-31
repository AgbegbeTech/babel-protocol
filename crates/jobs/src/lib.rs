use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AsyncJob {
    pub id: Uuid,
    pub idempotency_key: String,
    pub job_type: String,
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub attempt_count: u32,
    pub privacy_classification: String,
    pub authorized_references: Vec<String>,
}

impl AsyncJob {
    pub fn new(
        idempotency_key: impl Into<String>,
        job_type: impl Into<String>,
        privacy_classification: impl Into<String>,
        authorized_references: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            idempotency_key: idempotency_key.into(),
            job_type: job_type.into(),
            schema_version: 1,
            created_at: Utc::now(),
            attempt_count: 0,
            privacy_classification: privacy_classification.into(),
            authorized_references,
        }
    }
}
