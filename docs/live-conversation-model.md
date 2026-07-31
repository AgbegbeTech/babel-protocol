# Live Conversation Model

The room is the product.

A room supports participants, preferred languages, translation targets, presence, typing, ordered messages, durable acknowledgments, translation state, clarification, cultural context, repair, AI facilitation state, privacy settings, retention settings, artifact proposal permission, and closure.

Room lifecycle:

```text
created
active
paused
closed
archived
```

Closing a room does not publish, summarize, index, train on, or market its contents.

Delivery states:

```text
local_pending
received_by_edge
validating
persisted
delivered
translation_pending
translated
translation_reviewed
failed
```

Optimistic local echo is visually temporary. Durable state begins only after backend persistence.
