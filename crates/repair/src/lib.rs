use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairReason {
    Misunderstood,
    TranslationInaccurate,
    MissingContext,
    HarmfulWording,
    DisagreeWithInterpretation,
    NeedsClarification,
    PreserveOriginalWording,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairState {
    Open,
    Acknowledged,
    Clarifying,
    Resolved,
    Unresolved,
    Withdrawn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepairThread {
    pub id: Uuid,
    pub room_id: Uuid,
    pub target_id: String,
    pub opened_by: String,
    pub reason: RepairReason,
    pub note: String,
    pub state: RepairState,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RepairThread {
    pub fn open(
        room_id: Uuid,
        target_id: impl Into<String>,
        opened_by: impl Into<String>,
        reason: RepairReason,
        note: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            room_id,
            target_id: target_id.into(),
            opened_by: opened_by.into(),
            reason,
            note: note.into(),
            state: RepairState::Open,
            opened_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, state: RepairState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
}
