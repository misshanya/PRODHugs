# Backend

Go API server for PRODHugs.

## Tech Stack

- **Go 1.25** with [Echo v4](https://echo.labstack.com)
- **PostgreSQL** via [pgx/v5](https://github.com/jackc/pgx), [sqlc](https://sqlc.dev), [goose](https://github.com/pressly/goose) migrations
- **OpenAPI-first** code generation with [oapi-codegen](https://github.com/oapi-codegen/oapi-codegen)
- **WebSocket** for real-time events (hug completions, inbox notifications, online presence)

## Architecture

```
transport/http  ->  service  ->  repository  ->  db/sqlc/storage
```

- **Domain models** in `internal/models/`
- **Sentinel errors** in `internal/errorz/` — mapped to HTTP status codes via `errors.Is()`
- **Transactions** via `repository.NewTransactor(db).RunInTx(ctx, fn)`
- **Auth context** set by middleware: `UserIDContextKey` (uuid.UUID), `UserRoleContextKey`

## Project Structure

```
api/openapi.yaml                # OpenAPI spec (source of truth for HTTP API)
cmd/main.go                     # Entry point
internal/
  app/app.go                    # Component initialization and wiring
  config/                       # Config structs, loaded from .env + OS env
  db/
    migrations/                 # Goose SQL migrations (auto-run on startup)
    sqlc/                       # sqlc config, queries, and generated storage code
  errorz/                       # Domain-level sentinel errors
  models/                       # Domain models
  repository/                   # Data access layer
  service/                      # Business logic
  transport/http/               # HTTP handlers, middleware, generated OpenAPI code
  ws/                           # WebSocket hub
```

## Commands

| Task | Command |
|------|---------|
| Build | `go build -o ./tmp/main ./cmd` |
| Run | `go run ./cmd` |
| Dev (hot-reload) | `air` |
| Dev (Docker) | `docker compose -f compose-dev.yml up` (from repo root or `backend/`) |
| Lint | `golangci-lint run` |

## Code Generation

**Generated files (do not hand-edit):**
- `internal/transport/http/v1/api.gen.go`
- `internal/db/sqlc/storage/`

**Regenerate after changes:**

```sh
# After editing api/openapi.yaml
oapi-codegen -config oapi-codegen.yml api/openapi.yaml

# After editing SQL queries or migrations
sqlc generate -f internal/db/sqlc/sqlc.yaml
```

## Database

### New Migration

```sh
goose -dir internal/db/migrations create -s {name} sql
```

Migrations run automatically on app startup via embedded goose.

### New Query

1. Write SQL with sqlc annotations in `internal/db/sqlc/queries/`
2. Regenerate: `sqlc generate -f internal/db/sqlc/sqlc.yaml`
3. Use generated methods in the repository layer

## Environment

Required env vars (loaded from `.env`, then OS env):

| Variable | Description |
|----------|-------------|
| `POSTGRES_URL` | Full PostgreSQL connection string |
| `JWT_SECRET` | HS256 signing key for JWT tokens |

```sh
cp .env.example .env
```

## API

- **Swagger UI**: `/api/v1/swagger/`
- **OpenAPI spec**: `/api/v1/openapi.json`
- **WebSocket**: `/api/v1/ws` (authenticated, defined outside OpenAPI spec)

## Dev Tools

A Nix flake (`.envrc` / `flake.nix`) provides all dev tools: `sqlc`, `goose`, `oapi-codegen`, `air`, `golangci-lint`. Without Nix, install them manually.
