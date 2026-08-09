---
paths: ['frontend/**/*.ts', 'frontend/**/*.tsx', '**/package.json', '**/tsconfig.json']
---

# TypeScript / React 19 — universal-agent-runtime

Loaded when a frontend file is read. Not resident.

| Tier | Command |
|---|---|
| T0 every edit | `pnpm typecheck`; `pnpm lint` |
| T1 unit complete | `pnpm -C frontend test <file_pattern>` |
| T2 phase complete | `pnpm build`; `pnpm test` |
| T3 milestone only | e2e; visual regression |

Install with `pnpm -C frontend install --frozen-lockfile`. TypeScript 5.9.3,
2-space indent, semicolons. React code lives in `frontend/src/`.

`pnpm typecheck` is the real type gate — bundlers strip types without checking
them, so a green build proves nothing about types.

## Strict layering — do not skip a layer

1. **Components** never call `fetch`, never import Zustand stores directly, and
   never import `frontend/src/services/`. They render UI and call hooks.
2. **Hooks** subscribe to stores and expose store actions. They do not call
   `fetch` and do not import service modules.
3. **Stores** (`frontend/src/stores/`) hold state and call services for HTTP,
   SSE, and other I/O.
4. **Services** (`frontend/src/services/`) are thin wrappers around `fetch` and
   streams. **Only stores import services** — not hooks, not components.

This keeps data logic in one place, satisfies the `react-hooks/*` lint rules,
and keeps tests straightforward. A component that reaches past its layer will
pass review by looking correct and fail later by duplicating state.

## UI contract

React 19 is the authoritative first-party UI. Historical HTMX and Web Component
material in this repo is not present-tense product guidance — do not treat it as
a pattern to extend.

## Structure

`src/` is the Axum runtime and API. `frontend/` is React 19 and TypeScript.
`static/` holds bundled production assets. Configuration lives in `.env` (see
`.env.example`), `example.config.yaml`, and `mcp.json` for MCP tools.
