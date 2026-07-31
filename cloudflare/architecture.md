# Cloudflare Architecture

## Durable Object Responsibilities

One Durable Object per room:

```text
conversation:{room_id}
```

Responsibilities:

- maintain active WebSocket connections
- coordinate presence
- coordinate typing state
- route live events
- maintain short reconnect cursors
- apply temporary backpressure
- forward durable events to the Rust node
- broadcast only accepted persisted events

Durable Objects must not permanently store complete conversation history.

## Edge Worker Responsibilities

- request IDs
- security headers
- CORS
- request-size limits
- Turnstile verification
- routing
- basic envelope validation
- public response caching only
- rate limiting
- health responses

The Worker must not approve, publish, sign, infer consent, or authorize private content independently.

## Tunnel

Cloudflare Tunnel reaches the Rust node without opening a public inbound port.

Staging example:

```bash
cloudflared tunnel create babel-protocol-staging
cloudflared tunnel route dns babel-protocol-staging api.babel.ing
cloudflared tunnel run babel-protocol-staging
```

Store tunnel credentials outside the repository. Rotate if leaked.

## R2

R2 is used behind a provider-neutral object storage interface for encrypted private attachments, approved public artifacts, exports, collaboration deliverables, and public documentation.

R2 is not the only supported storage provider.

## Queues

Jobs include:

```text
job ID
idempotency key
job type
schema version
created timestamp
attempt count
privacy classification
authorized references
```

Queue payloads should not include full transcripts.
