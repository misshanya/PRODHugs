FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache deps separately.
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs \
 && cargo fetch \
 && rm -rf src

# Pre-touch the migrations dir so `sqlx::migrate!` macro can be parsed even
# before the real `migrations/` is copied (cargo check needs it).
RUN mkdir migrations && touch migrations/.keep

COPY . .

RUN cargo build --release --bin prodhugs

FROM debian:bookworm-slim AS runner
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/prodhugs /app/prodhugs
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/api /app/api

EXPOSE 8080 9090
USER 1000:1000
ENTRYPOINT ["/app/prodhugs"]
