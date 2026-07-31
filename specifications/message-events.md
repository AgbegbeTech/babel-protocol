# Message Events

Every live message preserves:

```text
message ID
room ID
sender identity
sender device
original language
original text
sent timestamp
client sequence
optional reply reference
signature
event hash
```

Translations, corrections, clarifications, repair requests, and cultural notes are separate child events.
