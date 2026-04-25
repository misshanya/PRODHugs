# AGENTS.md — Frontend

Vue 3.5 + TypeScript 6 + Vite 8 + Bun + Pinia 3 + Vue Router 5 + Tailwind CSS v4 + shadcn-vue 2 (reka-nova style).

See root `AGENTS.md` for full-stack context and Docker Compose instructions.

## Commands

All commands run from `frontend/`. Package manager is **Bun** — never use npm/yarn/pnpm.

| Task | Command |
|------|---------|
| Dev server | `bun dev` (port 3000, proxies `/api` to `localhost:8080`) |
| Build (type-check + vite) | `bun run build` |
| Build without type-check | `bun run build-only` |
| Type-check only | `bun run type-check` (`vue-tsc --build`) |
| Lint | `bun lint` (oxlint then eslint, both `--fix`) |
| Format | `bun format` (prettier) |

- `bun run build` runs type-check and vite build **in parallel** via `npm-run-all2`.
- Lint runs **two linters sequentially**: oxlint (with unicorn, vue, oxc plugins) first, then eslint. They coordinate via `eslint-plugin-oxlint` to avoid duplicate rules.
- No tests exist. No test runner is configured.

### Verification order

```
bun lint && bun run type-check && bun run build-only
```

## Code style

- **Prettier**: no semicolons, single quotes, 100 char print width.
- Path alias: `@/` maps to `src/`.
- `noUncheckedIndexedAccess` is enabled in `tsconfig.app.json` — indexed access returns `T | undefined`.
- UI strings are in **Russian**. Keep all user-facing text in Russian.

## Source layout

```
src/
  main.ts              # app entry — mounts Pinia, Router; imports vue-sonner/style.css
  App.vue              # conditional layout: sidebar + header for auth'd users, bare RouterView for guests
  api/client.ts        # axios instance (baseURL: /api/v1), all API methods, auth interceptors
  stores/auth.ts       # auth state, token/user in localStorage, login/register/logout
  stores/hugs.ts       # domain store — hugs, balance, leaderboard, user search/profile
  router/index.ts      # all routes; meta.auth = requires login, meta.guest = login/register only
  lib/utils.ts         # cn() helper (clsx + tailwind-merge)
  lib/validation.ts    # client-side validation mirroring backend rules + backend error parsing
  components/          # app-specific components (AppSidebar, AppHeader, HugButton, etc.)
  components/ui/       # shadcn-vue primitives (copy-pasted, directly editable)
  views/               # route-level pages
  composables/         # (empty, but aliased in components.json)
  assets/main.css      # Tailwind v4 imports, shadcn theme vars (oklch), custom animations
```

## Key patterns

- **API client** (`src/api/client.ts`): single axios instance at `/api/v1`. Request interceptor attaches `Bearer` token from `localStorage`. Response interceptor on 401 clears auth and redirects to `/login`. All API methods (auth, hugs, balance, users, leaderboard) are exported from this file.
- **Auth**: JWT token and user object stored in `localStorage` under keys `token` and `user`. Router guard in `router/index.ts` checks `meta.auth`/`meta.guest` against token presence.
- **Stores** use Pinia composition API (`defineStore` with setup function), not options API.
- **Validation** (`lib/validation.ts`): mirrors backend constraints (username 3–32 chars, password 8–128 chars with letter+digit+special). Also parses backend error responses (handler errors with `code`/`message`, OpenAPI validation errors with `type`/`detail`).

## Tailwind & shadcn-vue

- **Tailwind CSS v4** via Vite plugin (`@tailwindcss/vite`), not PostCSS. No `tailwind.config.js`.
- Theme variables defined as CSS custom properties in `src/assets/main.css` using oklch colors. Dark mode variant: `@custom-variant dark (&:is(.dark *))`.
- **Dark mode is always on** — `<html class="dark">` is hardcoded in `index.html`.
- **shadcn-vue** style is `reka-nova`, base color `neutral`, icon library `lucide` (`lucide-vue-next`). Components live in `src/components/ui/` — edit them directly, they are not imported from a package.
- **Adding new shadcn components**: `bunx shadcn-vue add <component>` — config is in `components.json`.
- **vue-sonner v2**: toast styles must be imported as `import 'vue-sonner/style.css'` in `main.ts` (already done). CSS `@import` from stylesheets does not work with this package.

## Docker

- **Prod Dockerfile** (`Dockerfile`): builds with `bun run build-only` (skips type-check), serves via nginx with SPA fallback.
- **Dev Dockerfile** (`Dockerfile.dev`): installs deps, source is volume-mounted, runs `bun dev --host 0.0.0.0`.
