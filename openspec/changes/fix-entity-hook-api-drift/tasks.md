## 1. Shared helper

- [x] 1.1 `useGraphEntities<T>(type)` + `useGraphEntity<T>(type, id)`
      (`frontend/src/entities/hooks/use-graph-entities.ts`).

## 2. Migrate broken call sites

- [x] 2.1 `use-models.ts` -> `useGraphEntities<ModelEntity>("Model")`
- [x] 2.2 `use-compiler-sessions.ts` -> `useGraphEntities<UarCompilerSession>("CompilerSession")`
- [x] 2.3 `use-mcp-status.ts` -> `useGraphEntities<McpStatusRow>("McpStatus")`
- [x] 2.4 `use-memory.ts` -> `useMemory()` via `useGraphEntities`, `useMemoryStats()`
      via `useGraphEntity`
- [x] 2.5 `use-settings-entity.ts` -> `useGraphEntity<SettingsNamespaceRow>`;
      also fixed `r.value` -> `r.data` (adjacent bug, same function)
- [x] 2.6 `tools-page.tsx` (inline call, not a dedicated hook file) ->
      `useGraphEntities<ToolWithNs>("Tool")`

## 3. Fix surfaced by verifying this fix

- [x] 3.1 `CompareDialog` (CH-10) width override `max-w-3xl` -> `sm:max-w-3xl`
      (tailwind-merge modifier mismatch left the base `sm:max-w-md` in
      effect, rendering the dialog at 448px instead of 768px).

## 4. Verify

- [x] 4.1 `bun run typecheck` — all hook-shape errors gone (17 unrelated
      pre-existing errors remain, documented in proposal.md).
- [x] 4.2 `bun run build` green.
- [x] 4.3 Live server smoke test: models catalog populates (5331/5331,
      150 providers, was 0 before); CH-10 compare dialog exercised
      end-to-end with real catalog data, both columns render correctly,
      zero console errors.
- [x] 4.4 `cargo test --lib` 330/330 (backend unaffected).
