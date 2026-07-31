# Threat Model

Threats considered in v0.1:

- stolen device keys
- malicious room participants
- malicious node operators
- forged messages
- forged consent
- unauthorized room access
- replay attacks
- message reordering
- translation manipulation
- prompt injection
- AI overreach
- metadata leakage
- compromised Cloudflare credentials
- Tunnel compromise
- R2 exposure
- queue duplication
- abusive repair requests
- coercive consent
- power imbalance
- spam
- denial of service
- unauthorized public indexing

Current mitigations include signed protocol envelopes, per-device sequence checks, room participant checks, no-store private responses, public API transcript tests, revision hashes for consent, and explicit AI boundaries.

Known limitation: v0.1 does not implement end-to-end encryption. E2EE and MLS are future protocol work.
