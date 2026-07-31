use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionPreference {
    Named,
    Pseudonymous,
    Anonymous,
    CommunityAttribution,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationScope {
    Private,
    Room,
    Commons,
    Public,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsentReceipt {
    pub receipt_id: Uuid,
    pub artifact_id: Uuid,
    pub exact_revision_hash: String,
    pub approving_participant: String,
    pub approving_device: String,
    pub approved_publication_scope: PublicationScope,
    pub attribution_preference: AttributionPreference,
    pub approved_translations: Vec<String>,
    pub ai_processing_permissions: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub optional_review_date: Option<DateTime<Utc>>,
    pub consent_statement_version: String,
    pub signature: String,
}

pub fn revision_hash(exact_text: &str) -> String {
    hex::encode(Sha256::digest(exact_text.as_bytes()))
}

impl ConsentReceipt {
    pub fn approves_revision(&self, revision_text: &str) -> bool {
        self.exact_revision_hash == revision_hash(revision_text)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{revision_hash, AttributionPreference, ConsentReceipt, PublicationScope};

    #[test]
    fn consent_is_revision_specific() {
        let text = "Shared insight revision one";
        let receipt = ConsentReceipt {
            receipt_id: Uuid::new_v4(),
            artifact_id: Uuid::new_v4(),
            exact_revision_hash: revision_hash(text),
            approving_participant: "did:babel:amara".to_string(),
            approving_device: "device-amara".to_string(),
            approved_publication_scope: PublicationScope::Commons,
            attribution_preference: AttributionPreference::Named,
            approved_translations: vec!["en".to_string(), "es".to_string()],
            ai_processing_permissions: vec!["artifact_draft".to_string()],
            timestamp: chrono::Utc::now(),
            optional_review_date: None,
            consent_statement_version: "babel-consent/1".to_string(),
            signature: "demo-signature".to_string(),
        };

        assert!(receipt.approves_revision(text));
        assert!(!receipt.approves_revision("Shared insight revision two"));
    }
}
