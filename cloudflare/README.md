# Cloudflare Reference Deployment

Cloudflare is mandatory for the hosted reference deployment, but not for local development.

Components:

- `edge-gateway`: security headers, CORS, request size checks, Turnstile placeholder, basic envelope validation, routing.
- `realtime-room`: one Durable Object per room, named `conversation:{room_id}`, for active WebSocket coordination.
- `queue-consumer`: minimum-necessary queue payload checks.

Cloudflare provides global reach, protection, and realtime coordination. Babel Protocol provides identity, consent, knowledge integrity, and portability. Cloudflare may host the network, but it does not define or own the protocol.

## Deploy

```bash
cd cloudflare/workers/realtime-room
wrangler deploy

cd ../edge-gateway
wrangler deploy

cd ../queue-consumer
wrangler deploy
```

## Required Cloudflare Products

- Workers
- Durable Objects
- WebSockets
- R2
- Queues
- Turnstile
- Tunnel
- DNS
- WAF
- rate limiting

Do not commit Cloudflare credentials, Tunnel tokens, API tokens, account identifiers, or production secrets.
