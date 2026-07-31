use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationRequest {
    pub message_id: String,
    pub source_language: String,
    pub target_language: String,
    pub original_text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TranslationResult {
    pub message_id: String,
    pub translated_text: String,
    pub source_language: String,
    pub target_language: String,
    pub provider: String,
    pub confidence: f32,
    pub uncertain_phrases: Vec<String>,
    pub literal_alternative: Option<String>,
    pub cultural_notes: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub review_status: TranslationReviewStatus,
    pub stream_state: TranslationStreamState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationReviewStatus {
    Unreviewed,
    Challenged,
    Corrected,
    Reviewed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationStreamState {
    TranslationStarted,
    TranslationPartial,
    TranslationCompleted,
    TranslationFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("translation is disabled")]
    Disabled,
    #[error("source text is empty")]
    EmptyInput,
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(
        &self,
        input: TranslationRequest,
    ) -> Result<TranslationResult, TranslationError>;
}

#[derive(Default)]
pub struct MockTranslationProvider;

#[async_trait]
impl TranslationProvider for MockTranslationProvider {
    async fn translate(
        &self,
        input: TranslationRequest,
    ) -> Result<TranslationResult, TranslationError> {
        if input.original_text.trim().is_empty() {
            return Err(TranslationError::EmptyInput);
        }

        let target = input.target_language.to_lowercase();
        let translated_text = if target.starts_with("es") {
            mock_spanish(&input.original_text)
        } else if target.starts_with("en") {
            mock_english(&input.original_text)
        } else {
            format!(
                "[mock {} translation] {}",
                input.target_language, input.original_text
            )
        };

        Ok(TranslationResult {
            message_id: input.message_id,
            translated_text,
            source_language: input.source_language,
            target_language: input.target_language,
            provider: "MockTranslationProvider".to_string(),
            confidence: 0.82,
            uncertain_phrases: vec![
                "local knowledge".to_string(),
                "outside organizations controlling the process".to_string(),
            ],
            literal_alternative: Some(
                "A literal translation may miss who holds authority in the project.".to_string(),
            ),
            cultural_notes: vec![
                "Review terms about community control with the speaker.".to_string(),
                "Water-project language may carry institutional history.".to_string(),
            ],
            generated_at: Utc::now(),
            review_status: TranslationReviewStatus::Unreviewed,
            stream_state: TranslationStreamState::TranslationCompleted,
        })
    }
}

#[derive(Default)]
pub struct NoTranslationProvider;

#[async_trait]
impl TranslationProvider for NoTranslationProvider {
    async fn translate(
        &self,
        _input: TranslationRequest,
    ) -> Result<TranslationResult, TranslationError> {
        Err(TranslationError::Disabled)
    }
}

fn mock_spanish(text: &str) -> String {
    if text.to_lowercase().contains("clean-water") || text.to_lowercase().contains("water") {
        "Podemos preservar el conocimiento local y coordinar proyectos de agua limpia de bajo costo sin que organizaciones externas controlen el proceso.".to_string()
    } else {
        format!("Traduccion simulada al espanol: {text}")
    }
}

fn mock_english(text: &str) -> String {
    if text.to_lowercase().contains("agua") || text.to_lowercase().contains("comunidad") {
        "The community should keep authority over local knowledge, decisions, and clean-water work."
            .to_string()
    } else {
        format!("Simulated English translation: {text}")
    }
}

#[cfg(test)]
mod tests {
    use super::{MockTranslationProvider, TranslationProvider, TranslationRequest};

    #[tokio::test]
    async fn mock_translation_keeps_uncertainty_visible() {
        let provider = MockTranslationProvider;
        let result = provider
            .translate(TranslationRequest {
                message_id: "msg-1".to_string(),
                source_language: "en".to_string(),
                target_language: "es".to_string(),
                original_text:
                    "How can local knowledge guide clean-water projects without outside control?"
                        .to_string(),
            })
            .await
            .unwrap();

        assert!(result.translated_text.contains("agua limpia"));
        assert!(!result.uncertain_phrases.is_empty());
        assert_eq!(result.provider, "MockTranslationProvider");
    }
}
