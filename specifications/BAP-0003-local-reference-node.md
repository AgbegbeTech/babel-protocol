# BAP-0003: Local Reference Node

## Status

Draft for Babel Protocol v0.1.

## Summary

The local reference node provides the same client-facing room protocol as the hosted Cloudflare path while running without a Cloudflare account.

## Rules

- Local mode must preserve original messages.
- Local mode must verify signatures for demo device events.
- Local mode must persist accepted messages before durable acknowledgment when PostgreSQL is configured.
- Local mode may use deterministic mock translation and facilitation.
- Local mode must not publish artifacts automatically.
