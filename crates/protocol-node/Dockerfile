FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p protocol-node

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/protocol-node /usr/local/bin/protocol-node
EXPOSE 8080
CMD ["protocol-node"]
