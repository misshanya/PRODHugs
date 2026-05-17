# AGENTS.md

## Project

Full-stack "PRODHugs" app — Rust backend + Vue 3 frontend, orchestrated via
Docker Compose behind nginx reverse proxy. No CI, no monorepo tooling —
just two directories with independent build systems.

## Repo layout

```
compose.yml / compose-dev.yml   # full-stack orchestration (nginx + frontend + backend + postgres)
nginx.conf / nginx-dev.conf     # reverse proxy: /api/ -> backend, everything else -> frontend
backend/                        # Rust (axum + sqlx + tokio) — has its own AGENTS.md
frontend/                       # Vue 3 + Vite 8 + Bun, shadcn-vue, Tailwind CSS v4
```

Backend has a detailed `backend/AGENTS.md` — read it when working in that
directory.

## Running the app

| Mode | Command (from repo root) | Access |
|------|--------------------------|--------|
| **Dev (full stack)** | `docker compose -f compose-dev.yml up` | `localhost:3001` (DEV_PORT) |
| **Prod (full stack)** | `docker compose up --build` | `localhost:3000` (APP_PORT) |
| **Frontend only** | `bun dev` in `frontend/` | `localhost:3000` (proxies `/api` to `:8080`) |
| **Backend only** | `cargo run` in `backend/` | `localhost:8080` |
| **Backend + DB only** | `docker compose -f compose-dev.yml up` in `backend/` | `localhost:8097` |

### Environment setup

Copy `.env.example` -> `.env` at repo root (used by root compose files).
Backend also needs its own `backend/.env` (copy from
`backend/.env.example`). Both need `JWT_SECRET` and Postgres credentials.

## Frontend (`frontend/`)

**Stack**: Vue 3.5 + TypeScript 6 + Vite 8 + Bun + Pinia 3 + Vue Router 5
+ Tailwind CSS v4 + shadcn-vue 2

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
- **`bun run build`** runs type-check and vite build in parallel via
  `npm-run-all2`. Use `bun run build-only` to skip type-check.
- **Lint runs two linters sequentially**: oxlint first, then eslint (both
  with `--fix`). They coordinate via `eslint-plugin-oxlint` to avoid
  duplicate rules.
- **Tailwind CSS v4** uses the Vite plugin (`@tailwindcss/vite`), not
  PostCSS. No `tailwind.config.js` exists.
- **shadcn-vue components** live in `src/components/ui/` — these are
  copy-pasted, not imported from a package. Edit them directly when
  needed.
- **vue-sonner v2** requires `import 'vue-sonner/style.css'` in `main.ts`
  for toast styling to work. CSS `@import` from stylesheets does not work
  with this package.
- **Vite dev server** proxies `/api` requests to `localhost:8080`
  (configured in `vite.config.ts`).
- **Prettier config**: no semicolons, single quotes, 100 char width.
- **Dark mode** is always on (`<html class="dark">` in `index.html`).
- **Auth**: token stored in `localStorage`, router guard checks
  `meta.auth` / `meta.guest`.
- No tests exist in the frontend.

## Backend (`backend/`)

See `backend/AGENTS.md` for full backend details. Key points:

- **Crate name**: `prodhugs` (binary in `src/main.rs`, library in
  `src/lib.rs`).
- **Architecture**: `http` → `service` → `repo` → `db (sqlx)`.
- **Migrations** are plain SQL files in `backend/migrations/`. `sqlx::migrate!`
  runs them on startup.
- **OpenAPI specs** in `backend/api/` are served verbatim at
  `/api/v1/openapi.json` and `/api/v2/openapi.json`.
- **Swagger UI** at `/api/v1/swagger/` (loaded from CDN, points at both
  specs).
- **WebSocket** endpoint at `/api/v1/ws`. The client must send
  `{"type":"auth","token":"<JWT>"}` as the first frame.
- **Hash format** — Argon2id matches the original Go service, so existing
  passwords still verify.

## Nginx routing

Both dev and prod nginx configs route:
- `/api/*` -> backend (with WebSocket upgrade support)
- Everything else -> frontend

Dev config additionally enables WebSocket passthrough for Vite HMR.

## What's unused / commented out

S3, Kafka, and Valkey: the original Go config structs are gone in the
Rust port. If we ever wire them up, do it through `src/config.rs` plus the
compose files.
