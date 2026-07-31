# Realtime Tests

Realtime scope:

- two connected clients receive accepted messages live
- messages are not durable before backend persistence
- reconnect restores missed persisted events
- typing state expires
- duplicate message events are rejected
- translation events attach to originals
- originals cannot be overwritten
