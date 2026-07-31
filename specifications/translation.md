# Translation

Translations are child events. They never replace original messages.

Provider interface:

```rust
#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(
        &self,
        input: TranslationRequest
    ) -> Result<TranslationResult, TranslationError>;
}
```

v0.1 includes `MockTranslationProvider` and `NoTranslationProvider`.

Translation results may include translated text, source language, target language, provider, confidence, uncertain phrases, literal alternative, cultural notes, generated timestamp, review status, and streaming state.

Streaming states:

```text
translation_started
translation_partial
translation_completed
translation_failed
```
