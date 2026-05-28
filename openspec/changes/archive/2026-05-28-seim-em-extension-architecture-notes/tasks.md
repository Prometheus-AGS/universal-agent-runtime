# Tasks: seim-em-extension-architecture-notes

## 1. Write architecture notes doc

- [x] 1.1 Create `docs/extension-architecture-notes.md` in seim-entity-management worktree covering:
  - 4-layer message bridge diagram (page → content → background → panel)
  - window.postMessage bridge design decision
  - MV3 constraints (ephemeral service worker, MAIN world injection)
  - File layout for W8 extension scaffold
  - How DevtoolsEventBus adapts for extension context

## 2. Add enableWindowBridge prop stub

- [x] 2.1 In `src/ui/entity-explorer/context.tsx`, add `enableWindowBridge?: boolean` to `EntityExplorerProviderProps`
- [x] 2.2 In `EntityExplorerProvider`, when `enableWindowBridge` is true, add a `useEffect` that:
  - Subscribes to the bus
  - On each event, calls `window.postMessage({ type: "__entity_explorer_event__", payload: event }, "*")`
  - Returns unsubscribe fn
- [x] 2.3 Run `pnpm test` — all tests pass

## 3. Commit

- [x] 3.1 `git add docs/extension-architecture-notes.md src/ui/entity-explorer/context.tsx`
- [x] 3.2 Commit: `docs(extension): Chrome MV3 architecture notes + window bridge stub (W7b)`

## 4. Archive

- [x] 4.1 Mark tasks done and archive
