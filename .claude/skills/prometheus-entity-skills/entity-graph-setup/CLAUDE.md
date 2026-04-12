# CLAUDE.md — entity-graph-setup skill

Skill-specific rules for Claude Code (and compatible agents) when executing **entity-graph-setup**.

## Before installing anything

1. **Detect existing state management** — Search for `@tanstack/react-query`, `@apollo/client`, `swr`, `redux`, `zustand` stores that own server entities, and raw `fetch` in components.
2. **Detect package manager** — Presence of `pnpm-lock.yaml`, `package-lock.json`, or `yarn.lock` determines install commands. **Never** mix managers in one change set.
3. **Read the app’s data-flow reality** — SSR (Next.js `fetch` in RSC), route handlers, and cookie auth affect where `configureEngine` and fetchers may run.

## Safety

- **Do not remove** legacy query hooks, reducers, or Apollo providers **without** a recorded backup (branch name, patch path, or git tag) in the migration plan.
- Prefer **parallel run** (new graph hooks behind a feature flag) over big-bang deletion unless the user explicitly requests cutover.
- **Do not** add `useGraphStore` imports to **component** files; only hooks and non-UI modules may touch the store directly per library rules.

## Code generation quality

- Generated fetchers must be **injected** into hooks via `configureEngine` / hook options—hooks themselves should not embed raw `fetch` URLs when the project already has a shared API client; **reuse** existing clients inside engine callbacks.
- Every new **public** hook wrapper in app code should include a short **JSDoc** if it becomes a shared internal API (mirrors library standards).
- After substantive edits, run **`pnpm run typecheck`** (or the project’s equivalent) from the correct package root.

## Monorepo note

When the workspace **is** @prometheus-ags/prometheus-entity-management, examples consume the library via path alias—**do not** add a redundant npm publish step for local verification.

## References

- `prometheus-entity-skills/_shared/references/architecture-rules.md` — non-negotiable layering.
- `prometheus-entity-skills/entity-graph-setup/references/migration-patterns.md` — strangler / parallel patterns.
- `prometheus-entity-skills/entity-graph-setup/references/codebase-analysis.md` — what to scan and in what order.
