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

## Required React/entity guidance

Before React or entity-state edits, read Vercel React Best Practices, Vercel
Composition Patterns, and the applicable Prometheus Entity Management skill.
Write one task-specific paragraph naming the rules applied before code changes.

## Strict layering — choose the state path, then do not skip a layer

1. **Components** render UI and call hooks. They never call `fetch`, import
   Zustand stores, services, transports, the entity-management package, or the
   raw graph store.
2. **Entity-backed path:** component → narrow platform domain hook → graph domain
   action → registered transport/API. Persistent, shared, server-confirmed, or
   domain-meaningful records use this path. Features import the domain only from
   `@/platform/entities`; feature code never performs raw or per-row graph writes.
3. **Transient path:** component → feature hook → Zustand store → service. This
   path is only for UI/workflow state that is not a business record. A transient
   store must not duplicate Provider, Model, AgentSession, AgentSessionDraft, or
   another graph-owned entity.
4. **Services/transports** own HTTP, SSE, and other external I/O. Hooks do not
   fetch. Components do not import service modules.

Use narrow selectors at the smallest rendered boundary. If two controls subscribe
to independent fields, split them into independent components; placing both hooks
in a sheet/page shell still rerenders the shell. React `useState` is allowed for
local widget mechanics such as open/closed state. Never call a setter in the
render body. Never ingest a fetched list with one graph mutation per row; use one
atomic graph ingestion action. A rerender is not automatically a bug—duplicated
business authority and broad subscriptions are.

## UI contract

React 19 is the authoritative first-party UI. Historical HTMX and Web Component
material in this repo is not present-tense product guidance — do not treat it as
a pattern to extend.

## Structure

`src/` is the Axum runtime and API. `frontend/` is React 19 and TypeScript.
`static/` holds bundled production assets. Configuration lives in `.env` (see
`.env.example`), `example.config.yaml`, and `mcp.json` for MCP tools.
