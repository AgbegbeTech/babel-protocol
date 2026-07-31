# Realtime Conversations

Client events:

```text
room.join
room.leave
presence.update
typing.start
typing.stop
message.send
message.clarification
translation.review
repair.open
repair.respond
facilitator.request
artifact.propose
```

Server events:

```text
room.snapshot
room.participant_joined
room.participant_left
message.accepted
message.rejected
message.delivered
translation.started
translation.partial
translation.completed
translation.failed
repair.updated
facilitator.response
artifact.proposal_created
error
```

Durable acceptance means the Rust node verified and persisted the original event. The edge may coordinate, but it must not independently authorize private content or mark messages durable.
