# UAR Frontend

> **Current authority:** [Chat product guide](/docs/product/chat). This README
> covers the first-party React source package; server-full and embedded-mobile
> packaging are verified separately.

The operator application uses React 19, TypeScript, Vite 8, Tailwind CSS 4,
Zustand, PGlite, and the Prometheus entity graph. The production web bundle is
written to `../static/` for the Axum `server-full` distribution. Tauri and
embedded-mobile packaging have separate host and platform boundaries; a web
build does not prove those targets.

## Ownership boundary

UI components render and dispatch intent. Hooks and view models adapt state.
Stores own business state. Services own REST and SSE I/O. Frontend code does
not call model providers, tools, or authoritative persistence directly.

SurrealDB remains authoritative for server entities. PGlite stores local
thread/message state and offline drafts. Server versions reconcile the entity
graph; unsent draft ownership remains local.

## Visual system

The shared tokens live in `src/shared/theme/tokens.css`, with staged HSL
compatibility variables in `src/index.css`. Dark, light, high-contrast, and
system themes are managed by `src/stores/theme-store.ts`. Product surfaces use
the Flat 2.0 rule: regions separate by color rather than decorative borders or
shadows, except where high contrast needs visible structure.

The first-party A2UI renderer is
`@prometheus-ags/a2ui-uar`. The Lit and Svelte packages are conformance
renderers; `@prometheus-ags/a2ui-react` is a private reference implementation.

## Local commands

Install from the repository root with the pinned pnpm version:

```bash
pnpm install --frozen-lockfile
pnpm -C frontend dev
pnpm -C frontend build
```

After a frontend unit is complete, its local checks are:

```bash
pnpm -C frontend typecheck
pnpm -C frontend lint
pnpm -C frontend test
```

GitHub Actions are deployment-only and do not run routine frontend checks.
