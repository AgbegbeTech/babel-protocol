# Protocol Versioning

Protocol identifier: `babel/1`.

Event schemas include their own semantic path and version, such as:

```text
babel.message.created/1
babel.message.translation/1
babel.message.translation_review/1
babel.message.clarification_request/1
babel.message.context_added/1
babel.message.repair_requested/1
```

Breaking changes require a new event schema version and a BAP. Provider-specific identifiers must not become protocol requirements.
