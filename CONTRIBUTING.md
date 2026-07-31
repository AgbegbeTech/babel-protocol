# Contributing

Thank you for helping build Babel Protocol.

Start by reading:

- `README.md`
- `ARCHITECTURE.md`
- `docs/privacy-model.md`
- `docs/philosophical-foundation.md`
- `specifications/BAP-0001-core-principles.md`
- `specifications/BAP-0002-dialogue-freedom-and-difference.md`

## Development

```bash
docker compose up --build
```

Use focused pull requests. Keep live conversation, privacy, consent, and participant agency at the center of the change.

## Protocol Changes

Use `specifications/BAP-TEMPLATE.md`. A BAP is required for changes that alter event schemas, consent rules, identity semantics, publication rules, or portability expectations.

## Security and Privacy

Never commit secrets, `.env`, `.dev.vars`, Cloudflare credentials, tunnel tokens, cookies, private keys, database exports, or private conversation content.

Do not add analytics containing message bodies. Do not log message bodies.
