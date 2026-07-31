use chrono::{DateTime, Utc};
use conversations::{CulturalContextNote, Message, Room};
use facilitation::FacilitationResponse;
use repair::{RepairReason, RepairState, RepairThread};
use serde::{Deserialize, Serialize};
use translation::TranslationResult;
use understanding::UnderstandingArtifactDraft;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomSnapshot {
    pub room: Room,
    pub messages: Vec<Message>,
    pub repairs: Vec<RepairThread>,
    pub facilitator_responses: Vec<FacilitationResponse>,
    pub artifact: Option<UnderstandingArtifactDraft>,
    pub approvals: Vec<String>,
    pub consent_receipt_ids: Vec<String>,
    pub commons_publications: Vec<PublicArtifactSummary>,
    pub projects: Vec<ProjectSummary>,
    pub server_time: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageSendInput {
    pub event: protocol_core::ProtocolEvent,
    pub original_language: String,
    pub original_text: String,
    pub reply_to: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationReviewInput {
    pub message_id: String,
    pub reviewer_id: String,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CulturalContextInput {
    pub message_id: String,
    pub author_id: String,
    pub note_type: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepairOpenInput {
    pub target_id: String,
    pub opened_by: String,
    pub reason: RepairReason,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepairTransitionInput {
    pub repair_id: String,
    pub state: RepairState,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FacilitationInput {
    pub requested_by: String,
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArtifactApprovalInput {
    pub participant_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicArtifactSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub revision_hash: String,
    pub consent_verified: bool,
    pub transcript_exposed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub source_artifact_id: String,
    pub contribution_needs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientEvent {
    #[serde(rename = "room.join")]
    RoomJoin { participant_id: String },
    #[serde(rename = "room.leave")]
    RoomLeave { participant_id: String },
    #[serde(rename = "presence.update")]
    PresenceUpdate {
        participant_id: String,
        present: bool,
    },
    #[serde(rename = "typing.start")]
    TypingStart { participant_id: String },
    #[serde(rename = "typing.stop")]
    TypingStop { participant_id: String },
    #[serde(rename = "message.send")]
    MessageSend(Box<MessageSendInput>),
    #[serde(rename = "translation.review")]
    TranslationReview(TranslationReviewInput),
    #[serde(rename = "message.context_added")]
    CulturalContext(CulturalContextInput),
    #[serde(rename = "repair.open")]
    RepairOpen(RepairOpenInput),
    #[serde(rename = "repair.respond")]
    RepairTransition(RepairTransitionInput),
    #[serde(rename = "facilitator.request")]
    FacilitatorRequest(FacilitationInput),
    #[serde(rename = "facilitator.reject")]
    FacilitatorReject { response_id: String },
    #[serde(rename = "artifact.propose")]
    ArtifactPropose { requested_by: String },
    #[serde(rename = "artifact.approve")]
    ArtifactApprove(ArtifactApprovalInput),
    #[serde(rename = "artifact.publish")]
    ArtifactPublish,
    #[serde(rename = "project.create")]
    ProjectCreate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerEvent {
    #[serde(rename = "room.snapshot")]
    RoomSnapshot(Box<RoomSnapshot>),
    #[serde(rename = "room.participant_joined")]
    ParticipantJoined { participant_id: String },
    #[serde(rename = "room.participant_left")]
    ParticipantLeft { participant_id: String },
    #[serde(rename = "message.accepted")]
    MessageAccepted(Box<Message>),
    #[serde(rename = "message.delivered")]
    MessageDelivered { message_id: String },
    #[serde(rename = "message.rejected")]
    MessageRejected { reason: String },
    #[serde(rename = "translation.started")]
    TranslationStarted { message_id: String },
    #[serde(rename = "translation.completed")]
    TranslationCompleted(Box<TranslationResult>),
    #[serde(rename = "repair.updated")]
    RepairUpdated(RepairThread),
    #[serde(rename = "message.context_added")]
    ContextAdded(CulturalContextNote),
    #[serde(rename = "facilitator.response")]
    FacilitatorResponse(FacilitationResponse),
    #[serde(rename = "artifact.proposal_created")]
    ArtifactProposalCreated(Box<UnderstandingArtifactDraft>),
    #[serde(rename = "artifact.updated")]
    ArtifactUpdated(Box<UnderstandingArtifactDraft>),
    #[serde(rename = "commons.published")]
    CommonsPublished(PublicArtifactSummary),
    #[serde(rename = "project.created")]
    ProjectCreated(ProjectSummary),
    #[serde(rename = "presence.updated")]
    PresenceUpdated {
        participant_id: String,
        present: bool,
    },
    #[serde(rename = "typing.updated")]
    TypingUpdated {
        participant_id: String,
        typing: bool,
    },
    #[serde(rename = "error")]
    Error { message: String },
}
