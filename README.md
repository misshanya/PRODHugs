# PRODHugs

Full-stack social app for virtual hugs. Go backend + Vue 3 frontend, orchestrated via Docker Compose behind an nginx reverse proxy.

## Features

- Two-phase hug system (suggest + accept/decline)
- Real-time feed via WebSocket
- Coin-based economy (daily rewards, hug slot purchases, cooldown upgrades)
- Intimacy system with tiers between user pairs
- Leaderboards (individual and pair-based)
- Telegram login integration
- Admin panel with user management
- Observability via Grafana Cloud (metrics + logs)

## Tech Stack

| Layer | Stack |
|-------|-------|
| Frontend | Vue 3.5, TypeScript, Vite 8, Bun, Pinia, Tailwind CSS v4, shadcn-vue |
| Backend | Go 1.25, Echo v4, OpenAPI codegen, sqlc, goose migrations |
| Database | PostgreSQL 18 |
| Proxy | nginx |
| Observability | Grafana Alloy, Prometheus, Loki |

## Repository Layout

```
compose.yml / compose-dev.yml   # full-stack orchestration
nginx.conf / nginx-dev.conf     # reverse proxy config
backend/                        # Go API server
frontend/                       # Vue SPA
alloy-config.alloy              # Grafana Alloy config for metrics/logs
```

## Getting Started

### Prerequisites

- Docker & Docker Compose
- (For local dev without Docker) Go 1.25+, Bun, PostgreSQL

### Environment

```sh
cp .env.example .env
# Edit .env — set JWT_SECRET and Postgres credentials
```

Backend also needs its own env file:

```sh
cp backend/.env.example backend/.env
```

### Running

| Mode | Command | Access |
|------|---------|--------|
| **Production** | `docker compose up --build` | `localhost:3000` |
| **Development** | `docker compose -f compose-dev.yml up` | `localhost:3001` |
| **Frontend only** | `bun dev` in `frontend/` | `localhost:3000` |
| **Backend only** | `air` in `backend/` | `localhost:8080` |

### Nginx Routing

- `/api/*` proxied to backend (with WebSocket upgrade support)
- Everything else served by frontend

## API Documentation

Swagger UI is available at `/api/v1/swagger/` when the backend is running. OpenAPI spec at `/api/v1/openapi.json`.

## Sub-projects

- [Backend](./backend/README.md)
- [Frontend](./frontend/README.md)
