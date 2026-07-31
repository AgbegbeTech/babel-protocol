# Architecture

Babel Protocol v0.1 is a modular monolith.

The Rust node is the durable authority for identity, membership, signatures, replay protection, message persistence, artifact approval, Commons publication, and project creation. Cloudflare Durable Objects coordinate live WebSockets for hosted deployments, but Durable Objects are not the canonical message database.

## Realtime Hosted Message Path

```text
Participant writes message
        ↓
Browser signs message event
        ↓
WebSocket sends event to conversation Durable Object
        ↓
Durable Object performs basic envelope validation
        ↓
Durable Object forwards event to Rust Babel node
        ↓
Rust node verifies device, signature, membership, schema, and replay protection
        ↓
Rust node persists the original message in PostgreSQL
        ↓
Rust node returns a durable acknowledgment
        ↓
Durable Object broadcasts the accepted message to connected participants
        ↓
Translation job is created when enabled
        ↓
Translation update streams back into the live room
```

The browser may show optimistic local echo. It must not mark a message as durably sent until the Rust node confirms persistence.

## Local Fallback

Local mode uses the same client-facing WebSocket event protocol without Cloudflare:

- Axum WebSocket endpoint at `/api/v1/rooms/:id/ws`
- PostgreSQL persistence when `BABEL_DATABASE_URL` is set
- in-memory fallback for unit tests and low-friction development
- deterministic mock translation
- deterministic mock AI facilitation

## Crate Boundaries

- `protocol-core`: event envelope, canonical signing payload, hashing, verification, replay protection.
- `identity`: person/device key material and signing helpers.
- `conversations`: rooms, participants, original messages, delivery state.
- `translation`: provider interface, no provider, mock provider.
- `facilitation`: provider interface, no provider, mock provider.
- `repair`: misunderstanding and repair states.
- `understanding`: artifact drafts and proposal provider.
- `consent`: revision-specific consent receipt model.
- `commons`: approved public artifact representation.
- `collaboration`: project model and contribution types.
- `jobs`: minimum-necessary async job envelope.
- `storage`: provider-neutral object references.
- `audit`: metadata-only audit records.

## Data Rules

Original messages are never overwritten. Translations, cultural context, clarification, review, and repair are child records or child events. Public Commons records do not contain room transcripts, private message IDs, private prompts, Power Context, raw secrets, queue payloads, or internal authorization data.
