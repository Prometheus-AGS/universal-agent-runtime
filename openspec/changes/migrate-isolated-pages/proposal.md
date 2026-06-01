## Why

Six admin pages own entities that no cross-cutting consumer depends on: Knowledge, Memory, Auth, Compiler, Tools, MCP Health. Migrating them first proves the entity-mgmt pattern end-to-end with the lowest possible blast radius — if anything regresses, only that one page is affected.

## What Changes

For each page:

1. **Replace** the page's bespoke Zustand store + `useEffect`-driven fetch with `useEntityList(<type>)` (and `useEntity(<type>, id)` for detail panels).
2. **Wire mutations** via `useEntityCRUD(<type>)` where appropriate.
3. **Delete** the corresponding Zustand store file in the same commit.
4. **Update imports** in the page; remove store references.

Pages + entities:

- `admin/pages/knowledge-page.tsx` → `knowledge_base` + `knowledge_document`. Delete `stores/knowledge-admin-store.ts`.
- `admin/pages/memory-page.tsx` → `memory`. Delete `stores/memory-admin-store.ts`. Search remains a direct service call (not graph-backed).
- `admin/pages/auth-page.tsx` → `api_key` with `realtime: false`, `refetchOnMutation: true`. Delete `stores/auth-keys-store.ts`.
- `admin/pages/compiler-page.tsx` → `compiler_session`. Delete `stores/compiler-sessions-store.ts`.
- `admin/pages/tools-page.tsx` → `tool`. Delete `stores/tools-discovery-store.ts`.
- `admin/McpHealthPage.tsx` → `useEntity("mcp_status", "current")`. Delete `stores/mcp-health-store.ts`.

## Acceptance

- Each page visually identical to current behaviour.
- `git grep -E "use(Knowledge|Memory|AuthKeys|CompilerSessions|ToolsDiscovery|McpHealth)(Admin)?Store"` returns zero hits.
- Multi-tab test on the KB page: upload doc in tab A, list refreshes in tab B without manual refresh.
