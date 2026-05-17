# Backend

Rust API server for PRODHugs (port of the original Go service — same data
model, same wire format).

## Tech Stack

- **Rust** (stable, 1.80+) with [axum 0.8](https://docs.rs/axum) and
  [tokio](https://tokio.rs)
- **PostgreSQL** via [sqlx](https://github.com/launchbadge/sqlx) (built-in
  migrate)
- **JWT** via [jsonwebtoken](https://crates.io/crates/jsonwebtoken),
  passwords hashed with [argon2](https://crates.io/crates/argon2) (same PHC
  output the Go service produced, so existing hashes still verify)
- **WebSocket** via `axum::extract::ws`
- **Telegram bot** — minimal long-polling client built on `reqwest`

## Architecture

```
http  ->  service  ->  repo  ->  db (sqlx)
```

- **Domain models** in `src/models/`
- **Domain errors** in `src/error.rs` (`AppError` → `ApiError` HTTP mapping)
- **Transactions**: services own a `PgPool`; for transactional flows they
  `let mut tx = pool.begin().await?` and pass `&mut *tx` into the repo
  functions (which accept any `impl sqlx::PgExecutor`).
- **Auth**: `AuthUser`/`AdminAuth` extractors decode the bearer token and
  inject `user_id` + `role`.

## Project Structure

```
api/openapi.yaml             # OpenAPI v1 (source of truth, served at /api/v1/openapi.json)
api/openapi-v2.yaml          # OpenAPI v2 (notes, daily-status, "@username" lookups)
migrations/                  # sqlx migrations (auto-run on startup)
src/
  main.rs                    # entry point + signal handling
  lib.rs                     # App construction + background tasks
  config.rs                  # env-driven configuration
  error.rs                   # AppError + ErrorCode
  crypto.rs                  # argon2 hash/verify
  jwt.rs                     # HS256 access/refresh/captcha tokens
  cache.rs                   # in-memory TTL cache
  metrics.rs                 # prometheus collectors + axum middleware
  db.rs                      # Transactor helper
  sudoku.rs                  # captcha puzzle generator
  models/                    # domain types
  repo/                      # data access (free functions, `impl PgExecutor`)
  service/                   # business logic (user, hug, note)
  telegram/                  # bot client, link/login stores, notifier
  ws.rs                      # WebSocket hub
  http/                      # routers, middleware, dto, v1 + v2 handlers
```

## Commands

| Task | Command |
|------|---------|
| Build | `cargo build --release` |
| Run | `cargo run` |
| Dev (hot reload) | `cargo watch -x run` (or `bacon`) |
| Dev (Docker) | `docker compose -f compose-dev.yml up` |
| Lint | `cargo clippy --all-targets` |
| Format | `cargo fmt` |

## Database

### New Migration

Add a file under `migrations/` named `NNNN_description.sql` (sqlx will pick
it up by lexical order — keep the same `00001…00030` prefix scheme that's
already in place). Migrations run automatically on app startup via
`sqlx::migrate!`.

### New Query

Each repo function lives in `src/repo/*.rs` and takes `impl sqlx::PgExecutor`.
Add the SQL inline (the previous Go service kept queries in
`internal/db/sqlc/queries/` for sqlc — in Rust we lean on `sqlx::query_as`
directly).

## Environment

Required env vars (loaded from `.env`, then OS env):

| Variable | Description |
|----------|-------------|
| `POSTGRES_URL` | Full PostgreSQL connection string |
| `JWT_SECRET` | HS256 signing key (≥ 32 chars; weak/default values rejected) |

Other knobs: `SERVER_ADDR`, `METRICS_ADDR`, `POSTGRES_MAX_CONNS`,
`JWT_ACCESS_DURATION`, `JWT_REFRESH_DURATION`, `JWT_COOKIE_SECURE`,
`CORS_ALLOW_ORIGINS`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_BOT_USERNAME`.

```sh
cp .env.example .env
```

## API

- **Swagger UI**: `/api/v1/swagger/`
- **OpenAPI v1**: `/api/v1/openapi.json`
- **OpenAPI v2**: `/api/v2/openapi.json`
- **WebSocket**: `/api/v1/ws` (authenticated by sending
  `{"type":"auth","token":"<JWT>"}` as the first frame)
