# Babel

The internet connected the world. Babel is being built to help the world understand itself.

Babel is a live, privacy-first communication protocol that helps people speak across languages and cultures, repair misunderstandings, preserve shared knowledge by mutual consent, and organize global collaboration.

Babel is a live communication protocol that helps people understand one another across languages and cultures—and, when they choose, turn that understanding into shared knowledge and coordinated action.

## Status

Babel Protocol v0.1 is experimental. It has not received an independent security audit. Do not use it for high-risk production conversations without additional review, deployment hardening, and operational controls.

The current repository is live chat first. The Commons, Understanding Artifacts, and collaboration projects are downstream, optional flows. Babel does not automatically publish conversations. AI cannot publish knowledge. Babel is not a blockchain, has no token, and does not require wallets, gas, smart contracts, or cryptocurrency.

Cloudflare is the initial hosted edge, not the protocol authority. The local system works without Cloudflare.

## Why Babel?

The biblical Babel represents a fracture in shared human understanding. Babel Protocol explores the opposite possibility: not one language or one worldview, but live infrastructure that helps different people understand one another without surrendering their identities.

Influenced in part by bell hooks' work on dialogue, voice, engaged pedagogy, community, and freedom, Babel treats communication as participation rather than extraction. AI may translate and clarify, but people retain authority over their words, their meaning, and what becomes shared knowledge.

Babel begins with the conversation. The Commons only begins when the participants choose.

> We do not need to become the same. We need to understand one another well enough to save what we share.

## Product Truth

The default screen is Conversations. A Babel conversation is successful when people communicate privately across language and culture, even if no artifact is ever created.

The essential sequence is:

```text
Live communication
        ↓
Mutual understanding
        ↓
Optional consent
        ↓
Shared knowledge
        ↓
Coordinated action
```

The conversation belongs to the participants. The understanding belongs to humanity only when the participants choose to contribute it.

## Repository Layout

```text
apps/web                  React + Vite PWA
apps/node-console         Developer inspection helper
crates/protocol-core      Signed protocol events, hashing, replay protection
crates/protocol-node      Axum local node, WebSocket room, API surface
crates/identity           Person and device identities
crates/translation        Translation provider interface and mocks
crates/facilitation       AI facilitation provider interface and mocks
crates/repair             Misunderstanding repair model
crates/consent            Revision-specific consent receipts
crates/understanding      Understanding Artifact model and proposal provider
crates/commons            Approved public knowledge layer
crates/collaboration      Project model
cloudflare/workers        Hosted edge reference
migrations                PostgreSQL schema
specifications            Babel Advancement Proposals and protocol specs
docs                      Mission, privacy, deployment, threat model
```

## Run Locally

```bash
docker compose up --build
```

Then open:

```text
http://localhost:5173
```

The local node runs at:

```text
http://localhost:8080
```

Health check:

```bash
curl http://localhost:8080/api/v1/health
```

## Demo Flow

1. Open the room as Amara.
2. Open another browser window or switch the demo identity to Diego.
3. Send an English message from Amara.
4. Send a Spanish reply from Diego.
5. Watch original messages and translations appear separately.
6. Challenge a translation.
7. Add cultural context.
8. Open a repair request.
9. Invite the AI facilitator.
10. Reject or ignore the AI suggestion.
11. Continue chatting privately.
12. Explicitly propose an Understanding Artifact.
13. Approve the exact revision as both participants.
14. Publish it to The Commons.
15. Verify the Commons response does not expose the room transcript.
16. Convert the approved artifact into a collaboration project.

The identity switcher is labeled `Development Demo Only`.

## Local Commands

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
pnpm install
pnpm run frontend:typecheck
pnpm run frontend:lint
pnpm run frontend:test
pnpm run frontend:build
pnpm run worker:typecheck
pnpm run worker:test
```

## Cloudflare Domains

The repository is prepared for:

```text
babel.ing
app.babel.ing
api.babel.ing
commons.babel.ing
docs.babel.ing
node.babel.ing
```

Use placeholders until the domain is connected.

## Cloudflare Deployment Sketch

```bash
cd cloudflare/workers/realtime-room && wrangler deploy
cd ../edge-gateway && wrangler deploy
cd ../queue-consumer && wrangler deploy
```

Cloudflare provides global reach, protection, and realtime coordination. Babel Protocol provides identity, consent, knowledge integrity, and portability. Cloudflare may host the network, but it does not define or own the protocol.

## Known Limitations

- v0.1 does not claim end-to-end encryption.
- v0.1 uses deterministic mock translation and mock AI facilitation by default.
- The local reference room is one-to-one, with models shaped for future group rooms.
- Cloudflare Workers are reference scaffolds and require account-specific configuration before production use.
- The demo uses local development identities. Production passkeys, recovery, and hardware-backed keys are future work.

## License

Apache License 2.0.
