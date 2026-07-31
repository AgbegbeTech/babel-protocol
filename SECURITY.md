# Security Policy

Babel Protocol v0.1 is experimental and has not received an independent security audit.

Do not report vulnerabilities by opening a public issue with exploit details. Send a private report to the maintainers listed in `CODEOWNERS` or use GitHub private vulnerability reporting when the public repository enables it.

## Scope

In scope:

- forged messages
- forged consent
- unauthorized room access
- replay attacks
- revoked device bypass
- private transcript leakage
- prompt injection that causes AI overreach
- public API transcript exposure
- Cloudflare Worker bypass of backend authorization

Out of scope for v0.1:

- claims that Babel lacks end-to-end encryption, because v0.1 does not claim E2EE
- cryptocurrency or token issues, because Babel has no token

## Reporting

Include:

- affected commit
- reproduction steps
- expected and actual behavior
- privacy impact
- whether private data, consent, or publication rules are affected

Do not include real private conversations.
