use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use translation::TranslationResult;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomLifecycle {
    Created,
    Active,
    Paused,
    Closed,
    Archived,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState {
    LocalPending,
    ReceivedByEdge,
    Validating,
    Persisted,
    Delivered,
    TranslationPending,
    Translated,
    TranslationReviewed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Participant {
    pub id: String,
    pub display_name: String,
    pub preferred_language: String,
    pub translation_target: String,
    pub location_label: String,
    pub present: bool,
    pub typing: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Room {
    pub id: Uuid,
    pub title: String,
    pub lifecycle: RoomLifecycle,
    pub privacy: String,
    pub retention: String,
    pub participants: Vec<Participant>,
    pub artifact_proposal_allowed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub id: String,
    pub room_id: Uuid,
    pub sender_id: String,
    pub sender_device_id: String,
    pub original_language: String,
    pub original_text: String,
    pub sent_at: DateTime<Utc>,
    pub client_sequence: u64,
    pub reply_to: Option<String>,
    pub signature: String,
    pub event_hash: String,
    pub delivery_state: DeliveryState,
    pub translations: Vec<TranslationResult>,
    pub context_notes: Vec<CulturalContextNote>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CulturalContextNote {
    pub id: String,
    pub message_id: String,
    pub author_id: String,
    pub note_type: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

impl Room {
    pub fn demo() -> Self {
        Self {
            id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("valid uuid"),
            title: "Local knowledge and clean-water projects".to_string(),
            lifecycle: RoomLifecycle::Active,
            privacy: "private_by_default".to_string(),
            retention: "participant_controlled".to_string(),
            participants: vec![
                Participant {
                    id: "did:babel:amara".to_string(),
                    display_name: "Amara".to_string(),
                    preferred_language: "English / Yoruba".to_string(),
                    translation_target: "es".to_string(),
                    location_label: "Lagos, Nigeria".to_string(),
                    present: false,
                    typing: false,
                },
                Participant {
                    id: "did:babel:diego".to_string(),
                    display_name: "Diego".to_string(),
                    preferred_language: "Spanish / English".to_string(),
                    translation_target: "en".to_string(),
                    location_label: "Medellin, Colombia".to_string(),
                    present: false,
                    typing: false,
                },
            ],
            artifact_proposal_allowed: true,
            created_at: Utc::now(),
        }
    }
}
