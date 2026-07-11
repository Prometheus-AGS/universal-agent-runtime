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

- [x] 3.1 Design verified in-browser across all 4 console pages — clean,
      consistent layout; panels reuse the existing EmptyRuntimeState design.
      (Operator chose structural browser proof over a full formal impeccable
      audit; net change was wiring + swapping to existing components, not
      net-new visual design.)
- [x] 3.2 Browser-drove all 4 console pages (server on :3000, auth off, embedded
      SurrealKV). Verified: A2UI Surfaces shows 5 real schemas; Model Routing
      shows real gpt-4o decision; Provider Health/Memory/Artifacts/AG-UI show
      neutral empty states; NO "not yet wired" banners anywhere. Screenshots
      captured.
- [x] 3.3 tsc --noEmit clean; frontend pnpm build green; runtime-ingest tests
      7/7; full cargo build green (embeds frontend via build.rs). Backend Rust
      unchanged so lib tests unaffected (389/389 from prior run).

## 4. Spec + bookkeeping

- [x] 4.1 Base runtime-console-ux spec updated: replaced "Unbuilt Panels
      Disclose Honestly" with "Panels Are Wired To Live Backend Emission".
- [x] 4.2 Committed (code 5e38f8d + this finalize), archived to
      archive/2026-07-11-runtime-console-wire-or-remove/, phase state advanced
      to 5/9. Pushed.
