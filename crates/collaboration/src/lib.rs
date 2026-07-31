use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionType {
    Research,
    Translation,
    Design,
    Engineering,
    FieldTesting,
    Documentation,
    Mentorship,
    Equipment,
    Introductions,
    Data,
    LocalContext,
    CommunityTrust,
    CareWork,
    Funding,
    Other,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CollaborationProject {
    pub id: Uuid,
    pub source_artifacts: Vec<Uuid>,
    pub title: String,
    pub problem: String,
    pub affected_communities: Vec<String>,
    pub cultural_context: Vec<String>,
    pub desired_outcome: String,
    pub required_languages: Vec<String>,
    pub contribution_needs: Vec<ContributionType>,
    pub open_questions: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
