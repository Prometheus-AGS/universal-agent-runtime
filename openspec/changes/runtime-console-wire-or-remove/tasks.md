## 1. Backend runtime.* emission (Rust)

- [x] 1.1 AG-UI panel prerequisite: `agui.*` stream already flows to the store;
      wired frontend-side in 2.1 (no backend change needed).
- [x] 1.2 Memory Activity: WIRED FRONTEND-SIDE (revised approach) — the store
      already parses `agui.memory.recall`/`agui.memory.mutation`, so route those
      into `RuntimeMemoryEvent` via `ingestRuntimeEvent` rather than adding a
      duplicate backend `runtime.memory_event` emitter. Done in chat-stream-store.ts.
- [x] 1.3 Artifacts: WIRED FRONTEND-SIDE (revised approach) — route
      `agui.artifact`/`agui.artifact_input_request` into `RuntimeArtifact`.
      Done in chat-stream-store.ts. (No duplicate backend arm.)
- [x] 1.4 Execution Timeline: `runtime.step` already carries `step`+`status`;
      normalized in the display layer (TimelineRow) so rows render non-blank
      (title = "Step N", status finished→done icon). No event-schema change.
- [x] 1.5 Provider Health: WIRED via REST feed (revised approach) —
      runtime-console-feeds.ts polls `GET /api/uar/providers/health` and upserts
      `RuntimeProviderHealth`. Lower-risk than a new `NormalizedEvent` variant.
- [x] 1.6 Model Routing: WIRED via REST feed — polls `GET /api/uar/resolve-model`
      and upserts `RuntimeModelRouteDecision` (default resolution).
- [x] 1.7 A2UI Surfaces: WIRED via REST feed — polls `GET /api/uar/a2ui/schemas`
      and upserts `RuntimeA2uiSurface` (registered surface schemas).

## 2. Frontend wiring (thin)

- [x] 2.1 AG-UI Events: route semantic `agui.*` frames to `ingestAgUiEvent` in
      chat-stream-store.ts (excludes high-frequency token deltas). Typecheck green.
- [x] 2.2 `EVENT_TYPE_MAP` + ingest already handle every runtime type; REST
      feeds upsert entities directly. runtime-ingest tests 7/7 green.
- [x] 2.3 Removed all 6 `NotWiredRuntimeState` banners (→ neutral
      `EmptyRuntimeState`); deleted the unused helper + orphaned imports.
      Typecheck + build green.

## 3. UI/UX + verification

- [ ] 3.1 UI/UX routing pass (impeccable audit + critique; frontend-design /
      ux-designer / UI/UX Pro Max) on the console; apply polish.
- [ ] 3.2 Browser-drive EVERY panel with real/replayed data and capture proof
      (Cockpit stat tiles + Provider Health + Memory; Runs steps/artifacts/
      tools; Approvals; Protocols AG-UI/Model Routing/A2UI).
- [ ] 3.3 `cargo test --lib` + frontend build/typecheck + runtime-ingest/e2e
      green.

## 4. Spec + bookkeeping

- [ ] 4.1 Update runtime-console-ux spec delta: panels wired to live emission;
      narrow the honest-disclosure requirement to genuinely-quiet states.
- [ ] 4.2 Commit, push, (validate), archive; update phase state.
