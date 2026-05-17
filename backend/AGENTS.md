# AGENTS.md

## Project

Rust API backend for PRODHugs (`axum` + `sqlx` + `tokio`). Port of the
original Go service — preserves the wire format, JWT shape, database
schema, and Argon2 hash format, so it drops in for the existing frontend
and Postgres data.

## Repo layout

```
api/                  # OpenAPI v1/v2 specs (served at /api/v1+v2/openapi.json)
migrations/           # sqlx migrations (auto-run at startup, NNNN_name.sql)
src/
  main.rs lib.rs      # entrypoint + App wiring (services, background jobs)
  config.rs error.rs  # env config + AppError/ErrorCode
  jwt.rs crypto.rs    # JWT manager (HS256) + Argon2 hashing
  cache.rs metrics.rs # in-memory TTL cache + prometheus collectors
  db.rs               # Transactor helper (services drive transactions directly)
  sudoku.rs           # captcha puzzle generator
  models/             # domain types
  repo/               # data access (free functions, `impl PgExecutor`)
  service/            # business logic (user, hug, note) + callback wiring
  telegram/           # long-polling bot, notifier, link/login stores
  ws.rs               # WebSocket hub
  http/               # axum routers, middlewares, DTOs, v1+v2 handlers
Cargo.toml Dockerfile compose.yml compose-dev.yml
```

## Commands

| Task | Command |
|------|---------|
| Build | `cargo build --release` |
| Run | `cargo run` |
| Test | `cargo test` |
| Lint | `cargo clippy --all-targets` |
| Format | `cargo fmt` |
| Dev (Docker) | `docker compose -f compose-dev.yml up` |

## Adding a new endpoint

1. Add the route in `src/http/v1/mod.rs` (or `v2/mod.rs`) and a handler
   function next to it.
2. Reach into the service layer (`src/service/`) for the business logic;
   add a thin repo function (`src/repo/`) if you need a new query.
3. Update `api/openapi.yaml` / `api/openapi-v2.yaml` — these specs are
   served verbatim at `/api/v1+v2/openapi.json`.

## Adding a new SQL query

Repo functions live in `src/repo/*.rs` and take any
`impl sqlx::PgExecutor`, so they participate naturally in transactions
(`pool.begin().await?; repo::xxx(&mut *tx, …)`). Add a function, call it
from the service.

## Adding a new migration

Drop a file into `migrations/NNNN_description.sql` (keep the existing
numeric prefix scheme). `sqlx::migrate!` picks it up at startup.

## Architecture layers

`http` → `service` → `repo` → `db (sqlx)`

- **Domain errors** in `src/error.rs`. The HTTP layer maps them to status
  codes + stable `ErrorCode` strings in `src/http/error.rs`.
- **Transactions** are driven from the service layer:
  `let mut tx = self.pool.begin().await?; …; tx.commit().await?;`. Repo
  functions take `impl sqlx::PgExecutor`, so the same call site works with
  `&pool` or `&mut *tx`.
- **Auth**: `http::auth::AuthUser` is the extractor every authenticated
  handler depends on; `AdminAuth` enforces the admin role.

## Environment

Required env vars (loaded from `.env`, then OS env):
- `POSTGRES_URL` — full connection string
- `JWT_SECRET` — HS256 signing key (≥ 32 chars; weak/default values are
  rejected at startup)

Other knobs: `SERVER_ADDR`, `METRICS_ADDR`, `POSTGRES_MAX_CONNS`,
`JWT_ACCESS_DURATION`, `JWT_REFRESH_DURATION`, `JWT_COOKIE_SECURE`,
`CORS_ALLOW_ORIGINS`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_BOT_USERNAME`.

## Gotchas

- The Argon2 hash format matches the Go version
  (`$argon2id$v=19$m=65536,t=3,p=2$…`), so existing user passwords still
  verify after the migration.
- Migrations are sqlx-style plain SQL files in lexical order — the old
  `+goose Up / +goose Down` annotations are stripped.
- The Telegram bot uses a hand-rolled long-polling client over `reqwest`
  (no `teloxide` dependency). Empty `TELEGRAM_BOT_TOKEN` disables the bot
  and notifier cleanly.
- WebSocket auth handshake is unchanged: the first client frame must be
  `{"type":"auth","token":"<JWT>"}`.
- Prometheus metrics are served on a separate listener (`METRICS_ADDR`,
  default `:9090`).
