# Frontend

Vue 3 SPA for Hugs as a Service.

## Tech Stack

- **Vue 3.5** + TypeScript + **Vite 8**
- **Bun** as package manager and runtime
- **Pinia 3** for state management (composition API)
- **Vue Router 5** with auth guards
- **Tailwind CSS v4** via Vite plugin
- **shadcn-vue 2** (reka-nova style) for UI components
- **lucide-vue-next** for icons
- **vue-sonner** for toast notifications

## Commands

All commands run from `frontend/`. Package manager is **Bun**.

| Task | Command |
|------|---------|
| Dev server | `bun dev` (port 3000, proxies `/api` to `localhost:8080`) |
| Build | `bun run build` (type-check + vite build in parallel) |
| Build without type-check | `bun run build-only` |
| Type-check | `bun run type-check` |
| Lint | `bun lint` (oxlint then eslint, both with `--fix`) |
| Format | `bun format` (prettier) |

## Project Structure

```
src/
  main.ts              # App entry point
  App.vue              # Root layout (sidebar for auth'd users, bare view for guests)
  api/client.ts        # Axios instance, all API methods, auth interceptors
  stores/              # Pinia stores (auth, hugs, admin, online)
  router/index.ts      # Routes with meta.auth / meta.guest guards
  views/               # Route-level pages
  components/          # App-specific components
  components/ui/       # shadcn-vue primitives (copy-pasted, directly editable)
  composables/         # Shared composables (WebSocket, Telegram login)
  lib/utils.ts         # cn() helper (clsx + tailwind-merge)
  lib/validation.ts    # Client-side validation mirroring backend rules
  assets/main.css      # Tailwind v4 imports, theme variables, custom animations
```

## Key Patterns

- **API client** (`src/api/client.ts`): single axios instance at `/api/v1`. Request interceptor attaches Bearer token. Response interceptor clears auth on 401.
- **Auth**: JWT token and user object stored in `localStorage`. Router guard checks `meta.auth` / `meta.guest`.
- **Dark mode** is always on (`<html class="dark">` hardcoded in `index.html`).
- **UI strings** are in Russian.

## Tailwind & shadcn-vue

- Tailwind CSS v4 via Vite plugin (no `tailwind.config.js`).
- Theme variables in `src/assets/main.css` using oklch colors.
- shadcn-vue components in `src/components/ui/` -- edit them directly.
- Add new components: `bunx shadcn-vue add <component>`.

## Docker

- **Production**: `Dockerfile` -- builds with `bun run build-only`, serves via nginx with SPA fallback.
- **Development**: `Dockerfile.dev` -- volume-mounted source, runs `bun dev --host 0.0.0.0`.
