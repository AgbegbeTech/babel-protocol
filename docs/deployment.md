# Deployment

## Local

```bash
docker compose up --build
```

## Hosted Reference

1. Create PostgreSQL.
2. Configure the Rust node with `BABEL_DATABASE_URL`.
3. Connect Cloudflare Tunnel to the Rust node.
4. Deploy `cloudflare/workers/realtime-room`.
5. Deploy `cloudflare/workers/edge-gateway`.
6. Deploy `cloudflare/workers/queue-consumer`.
7. Configure DNS for `babel.ing`, `app.babel.ing`, `api.babel.ing`, `commons.babel.ing`, `docs.babel.ing`, and `node.babel.ing`.
8. Configure WAF, rate limits, Turnstile, Queues, and R2 buckets.

Never commit Cloudflare account IDs, tunnel tokens, API tokens, R2 credentials, or production host secrets.
