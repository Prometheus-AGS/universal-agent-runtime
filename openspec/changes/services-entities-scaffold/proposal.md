## Why

`@prometheus-ags/prometheus-entity-management`'s `useEntity` / `useEntityList` hooks expect host-supplied `fetch` + `normalize` callbacks. UAR has 13 entity types (providers, models, agents, skills, KBs, documents, settings, memory, api_keys, tools, compiler_sessions, mcp_status, threads). Each needs a thin entity-services module that re-exports a `fetchEntity` + `fetchList` (+ optional mutations) wrapping the existing transport in `services/*-api.ts`.

## What Changes

Add 13 thin modules under `frontend/src/services/entities/`, one per entity type:

```ts
// frontend/src/services/entities/provider.ts
import { listProviders, getProvider } from "@/services/providers-api";

export const PROVIDER = "provider" as const;

export async function fetchEntity(id: string) {
  return getProvider(id);
}

export async function fetchList() {
  return listProviders();
}

export function normalize(raw: ApiProvider): ProviderEntity {
  return { ...raw }; // identity normalization in most cases
}
```

Modules to create:
`provider.ts`, `model.ts`, `agent.ts`, `skill.ts`, `knowledge-base.ts`, `knowledge-document.ts`, `setting.ts`, `memory.ts`, `api-key.ts`, `tool.ts`, `compiler-session.ts`, `mcp-status.ts`, `thread.ts`.

Each module exports:
- A type-literal constant (`export const PROVIDER = "provider" as const`) used as the entity-mgmt graph key.
- `fetchEntity(id)` and `fetchList(params?)`.
- `normalize(raw)` that maps backend response to the entity shape stored in the graph.
- Optional `mutate*` helpers (POST/PUT/PATCH/DELETE) for use with `useEntityCRUD`.

**No transport rewrites in this change.** The existing `services/*-api.ts` modules stay; entity modules wrap them.

## Acceptance

- 13 files exist under `frontend/src/services/entities/`.
- `pnpm --filter ./frontend tsc --noEmit` passes.
- A demo `useEntityList("provider")` call in a throwaway component resolves data correctly.
