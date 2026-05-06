# AGENTS.md

## Project

Full-stack "PRODHugs" app — Go backend + Vue 3 frontend, orchestrated via Docker Compose behind nginx reverse proxy. No CI, no monorepo tooling — just two directories with independent build systems.

## Repo layout

```
compose.yml / compose-dev.yml   # full-stack orchestration (nginx + frontend + backend + postgres)
nginx.conf / nginx-dev.conf     # reverse proxy: /api/ -> backend, everything else -> frontend
backend/                        # Go 1.25, Echo v4, OpenAPI codegen — has its own AGENTS.md
frontend/                       # Vue 3 + Vite 8 + Bun, shadcn-vue, Tailwind CSS v4
```

Backend has a detailed `backend/AGENTS.md` — read it when working in that directory.

## Running the app

| Mode | Command (from repo root) | Access |
|------|--------------------------|--------|
| **Dev (full stack)** | `docker compose -f compose-dev.yml up` | `localhost:3001` (DEV_PORT) |
| **Prod (full stack)** | `docker compose up --build` | `localhost:3000` (APP_PORT) |
| **Frontend only** | `bun dev` in `frontend/` | `localhost:3000` (proxies `/api` to `:8080`) |
| **Backend only** | `air` in `backend/` (or `go run ./cmd`) | `localhost:8080` |
| **Backend + DB only** | `docker compose -f compose-dev.yml up` in `backend/` | `localhost:8097` |

### Environment setup

Copy `.env.example` -> `.env` at repo root (used by root compose files). Backend also needs its own `backend/.env` (copy from `backend/.env.example`). Both need `JWT_SECRET` and Postgres credentials.

## Frontend (`frontend/`)

**Stack**: Vue 3.5 + TypeScript 6 + Vite 8 + Bun + Pinia 3 + Vue Router 5 + Tailwind CSS v4 + shadcn-vue 2

| Task | Command (in `frontend/`) |
|------|--------------------------|
| Dev server | `bun dev` |
| Build (type-check + vite) | `bun run build` |
| Type-check only | `bun run type-check` (`vue-tsc --build`) |
| Lint (oxlint then eslint) | `bun lint` |
| Format | `bun format` |
| Build without type-check | `bun run build-only` |

### Quirks

- **Package manager is Bun**, not npm/yarn/pnpm. Lockfile is `bun.lock`.
- **`bun run build`** runs type-check and vite build in parallel via `npm-run-all2`. Use `bun run build-only` to skip type-check.
- **Lint runs two linters sequentially**: oxlint first, then eslint (both with `--fix`). They coordinate via `eslint-plugin-oxlint` to avoid duplicate rules.
- **Tailwind CSS v4** uses the Vite plugin (`@tailwindcss/vite`), not PostCSS. No `tailwind.config.js` exists.
- **shadcn-vue components** live in `src/components/ui/` — these are copy-pasted, not imported from a package. Edit them directly when needed.
- **vue-sonner v2** requires `import 'vue-sonner/style.css'` in `main.ts` for toast styling to work. CSS `@import` from stylesheets does not work with this package.
- **Vite dev server** proxies `/api` requests to `localhost:8080` (configured in `vite.config.ts`).
- **Prettier config**: no semicolons, single quotes, 100 char width.
- **Dark mode** is always on (`<html class="dark">` in `index.html`).
- **Auth**: token stored in `localStorage`, router guard checks `meta.auth` / `meta.guest`.
- No tests exist in the frontend.

## Backend (`backend/`)

See `backend/AGENTS.md` for full backend details. Key points:

- **Go module** is still named `go-service-template` — use that in imports.
- **Generated code — do not hand-edit**:
  - `internal/transport/http/v1/api.gen.go` (from `oapi-codegen`)
  - `internal/db/sqlc/storage/` (from `sqlc`)
- **Codegen commands** (run from `backend/`):
  - `oapi-codegen -config oapi-codegen.yml api/openapi.yaml` — after editing OpenAPI spec
  - `sqlc generate -f internal/db/sqlc/sqlc.yaml` — after editing SQL queries or migrations
- **New migration**: `goose -dir internal/db/migrations create -s {name} sql`
- **Migrations auto-run** on app startup (embedded goose).
- **Architecture**: `transport/http` -> `service` -> `repository` -> `db/sqlc/storage`
- **Nix flake** provides all dev tools (`sqlc`, `goose`, `oapi-codegen`, `air`, `golangci-lint`). Without Nix, install them manually.
- **No tests exist** — zero `_test.go` files.
- **Swagger UI** at `/api/v1/swagger/`, OpenAPI spec at `/api/v1/openapi.json`.
- **WebSocket** endpoint at `/api/v1/ws` — defined outside OpenAPI spec, registered directly in `app.go`.

## Nginx routing

Both dev and prod nginx configs route:
- `/api/*` -> backend (with WebSocket upgrade support)
- Everything else -> frontend

Dev config additionally enables WebSocket passthrough for Vite HMR.

## What's unused / commented out

S3, Kafka, and Valkey: config structs exist in `backend/internal/config/`, but initialization is commented out in `backend/internal/app/app.go` and the corresponding services are absent from compose files. Don't wire them up without uncommenting both sides.
