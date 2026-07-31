use async_trait::async_trait;
use chrono::{DateTime, Utc};
use consent::revision_hash;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactLifecycle {
    Draft,
    Proposed,
    UnderReview,
    Approved,
    Published,
    Withdrawn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnderstandingArtifactDraft {
    pub id: Uuid,
    pub room_id: Uuid,
    pub title: String,
    pub originating_question: String,
    pub shared_summary: String,
    pub cultural_contexts: Vec<String>,
    pub linguistic_ambiguities: Vec<String>,
    pub areas_of_agreement: Vec<String>,
    pub unresolved_disagreements: Vec<String>,
    pub minority_perspectives: Vec<String>,
    pub evidence_references: Vec<String>,
    pub approved_translations: Vec<String>,
    pub required_approvers: Vec<String>,
    pub ai_disclosure: String,
    pub publication_scope: String,
    pub revision_hash: String,
    pub lifecycle: ArtifactLifecycle,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InsightProposalInput {
    pub room_id: Uuid,
    pub requested_by: String,
    pub participant_ids: Vec<String>,
    pub visible_context: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InsightProposalError {
    #[error("artifact proposal is disabled")]
    Disabled,
    #[error("at least one participant is required")]
    MissingParticipants,
}

#[async_trait]
pub trait InsightProposalProvider: Send + Sync {
    async fn propose(
        &self,
        input: InsightProposalInput,
    ) -> Result<UnderstandingArtifactDraft, InsightProposalError>;
}

#[derive(Default)]
pub struct MockInsightProposalProvider;

#[async_trait]
impl InsightProposalProvider for MockInsightProposalProvider {
    async fn propose(
        &self,
        input: InsightProposalInput,
    ) -> Result<UnderstandingArtifactDraft, InsightProposalError> {
        if input.participant_ids.is_empty() {
            return Err(InsightProposalError::MissingParticipants);
        }

        let shared_summary = "Communities should preserve local knowledge, choose their own accountable stewards, and invite technical support only when it strengthens community control of low-cost clean-water work.".to_string();
        Ok(UnderstandingArtifactDraft {
            id: Uuid::new_v4(),
            room_id: input.room_id,
            title: "Community-led clean-water knowledge".to_string(),
            originating_question: "How can communities preserve local knowledge and coordinate low-cost clean-water projects without outside organizations controlling the process?".to_string(),
            shared_summary: shared_summary.clone(),
            cultural_contexts: vec![
                "Terms like control, stewardship, and outside help should be reviewed locally.".to_string(),
            ],
            linguistic_ambiguities: vec![
                "Community control may not map exactly across English and Spanish usage.".to_string(),
            ],
            areas_of_agreement: vec![
                "Local participants should approve what becomes public.".to_string(),
            ],
            unresolved_disagreements: vec![
                "The participants have not settled how technical experts should be selected.".to_string(),
            ],
            minority_perspectives: Vec::new(),
            evidence_references: Vec::new(),
            approved_translations: Vec::new(),
            required_approvers: input.participant_ids,
            ai_disclosure: "AI-proposed. Human-reviewed. Not yet approved.".to_string(),
            publication_scope: "commons".to_string(),
            revision_hash: revision_hash(&shared_summary),
            lifecycle: ArtifactLifecycle::Draft,
            created_at: Utc::now(),
        })
    }
}

#[derive(Default)]
pub struct NoInsightProposalProvider;

#[async_trait]
impl InsightProposalProvider for NoInsightProposalProvider {
    async fn propose(
        &self,
        _input: InsightProposalInput,
    ) -> Result<UnderstandingArtifactDraft, InsightProposalError> {
        Err(InsightProposalError::Disabled)
    }
}
