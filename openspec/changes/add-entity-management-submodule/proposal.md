## Why

`@prometheus-ags/prometheus-entity-management` is the canonical entity-graph + hooks library we want every Admin view to consume. Embedding it as a git submodule under `frontend/packages/prometheus-entity-management/` (rather than a published version) lets us iterate on both repos in lockstep and ship matching breaking changes atomically.

## What Changes

- `git submodule add git@github.com:Prometheus-AGS/prometheus-entity-management.git frontend/packages/prometheus-entity-management`.
- `git submodule update --init --recursive`.
- Add `"@prometheus-ags/prometheus-entity-management": "workspace:*"` to `frontend/package.json` `dependencies`.
- `pnpm --filter prometheus-entity-management build` produces `dist/index.mjs`.
- Vite/TSConfig alias verification — workspace protocol should resolve out of the box; if Vite needs an explicit alias to the built `dist/`, add it.
- No SPA wiring yet; that ships in `configure-entity-engine-and-realtime-bridge`.

## Acceptance

- `import { configureEngine } from "@prometheus-ags/prometheus-entity-management"` resolves and type-checks in the SPA without errors.
- `cargo build` chain (pnpm install → pnpm build → vite build) succeeds.
