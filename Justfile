set dotenv-load := false

node := "crates/protocol-node"
web := "apps/web"

fmt:
    cargo fmt --all
    pnpm exec prettier --write "apps/web/src/**/*.{ts,tsx,css}" "cloudflare/workers/**/*.ts" "packages/**/*.ts" || true

check:
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

dev:
    docker compose up --build

node-dev:
    cargo run -p protocol-node

web-dev:
    pnpm --dir {{web}} dev --host 0.0.0.0

migrate:
    sqlx migrate run --database-url ${BABEL_DATABASE_URL}
