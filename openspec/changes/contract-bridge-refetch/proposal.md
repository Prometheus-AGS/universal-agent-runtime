## Why

`useGraphBridge` is the on-ramp that keeps 8 still-bridged Zustand admin hooks fresh on SSE-fed graph mutations. Until each entity is direct-migrated, this helper is load-bearing — a regression here would silently re-introduce stale data across Knowledge, Memory, Compiler, Tools, MCP-Health, Models, Skills, Settings.

## What Changes

Author `frontend/src/lib/realtime/__tests__/use-graph-bridge.test.tsx`:

- Render a component that calls `useGraphBridge(["Provider"], loadSpy)` where `loadSpy = vi.fn()`.
- `useGraphStore.getState().upsertEntity("Provider", "p1", { id: "p1" })`.
- `await waitFor(() => expect(loadSpy).toHaveBeenCalledTimes(1))`.
- Mutate an *unrelated* type (`upsertEntity("Setting", "k1", …)`): `expect(loadSpy).toHaveBeenCalledTimes(1)` — no extra call.
- Multi-key watch test: `useGraphBridge(["Provider", "Model"], spy)`; mutating either fires once.

## Acceptance

- Test passes.
- Toggling the `useEffect`-driven subscription off in the helper makes the test fail.
