# @prometheus-ags/prometheus-entity-management — architecture rules (agent reference)

Non-negotiable constraints distilled from `CLAUDE.md` / `AGENTS.md`. Violating these breaks cross-view reactivity and the purpose of the normalized graph.

## Three-layer model

Data flows **up** into the graph; UI reads **down** through hooks.

| Layer | Location | Responsibility |
|-------|----------|----------------|
| **UI** | Components | Render only; call hooks for data and actions. |
| **Hooks** | `hooks.ts`, `graphql/hooks.ts`, `crud/`, `view/` | Orchestrate reads/writes between components and the graph; **no direct I/O**. |
| **Graph + engine** | `graph.ts`, `engine.ts`, stores/adapters | Own fetches, mutations, realtime, cache writes. |

Sideways data flow between components (duplicated caches, prop-drilled copies) is an anti-pattern; the graph is the single source of truth.

## Components NEVER interact directly with stores

- Do **not** import or subscribe to `useGraphStore` inside **component** files.
- Components use **`useEntity`**, **`useEntityList`**, **`useEntityView`**, **`useEntityCRUD`**, GraphQL hooks, etc.
- `useGraphStore.getState()` is allowed **inside** engine code, adapters, CRUD internals, **custom app hooks** (e.g. syncing TanStack Query results into the graph), effects, workers, and other **non-component** modules—not in presentational `*.tsx` components.

## Entities live exactly once (normalized cache)

- Upsert every server-shaped record into `entities[type][id]`; store **IDs** elsewhere (lists, relations, UI selection).
- Never keep a second copy of the same row in React state, context, or another cache unless it is a deliberate, short-lived edit buffer (see CRUD `editBuffer`).

## Queries are instructions, not containers

- Hooks describe **what** to load and **how** to normalize; they do not own the resulting data.
- The graph owns canonical data. Changing one entity updates every view that joins list IDs to the graph.

## Lists store IDs, not data

- `ListState.ids` is an ordered array of **`EntityId`** values plus pagination meta.
- Row objects at render time = `ids.map(id => readEntity(type, id))` (via hooks). This enables instant cross-screen updates when any entity changes.

## Immer for all graph mutations

- All writes that touch `entities`, `patches`, `lists`, or `entityStates` go through the Zustand + Immer middleware paths provided by the store API (`upsertEntity`, `patchEntity`, etc.).
- Do not mutate graph state with plain assignment from app code.

## JSDoc on public hooks

- Every **exported** hook in the library carries JSDoc describing purpose, parameters, and the problem it solves. New public hooks must follow the same standard.

## TypeScript: avoid `any`

- No `any` except at unavoidable **adapter boundaries** (e.g. wrapping third-party clients). When used, document **why** in a short comment.
- Prefer `unknown`, generics, and `Record<string, unknown>` for entity-shaped data.

## Package manager (monorepo)

- **`pnpm` only** for this monorepo and for skills that scaffold into repos that adopt the same rule.

## Related mental models

- **Patches** (`patches[type][id]`) are UI-only overlays merged at read time; never sent to the server.
- **Cascade invalidation** after mutations uses `registerSchema` metadata to mark related lists/entities stale—prefer this over manual key hunting.
