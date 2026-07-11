## Why

The Runtime Console (admin shell, `/admin/*`) has four pages (Cockpit, Runs,
Approvals, Protocols) whose panels render from a normalized runtime entity
graph. Six panels currently show honest "not yet wired" disclosures —
Provider Health, Memory Activity, Artifacts, AG-UI Events, Model Routing,
A2UI Surfaces — because the backend has no `runtime.*` emission path for their
entity types (`runtime-console-ux` spec, "Unbuilt Panels Disclose Honestly").

Operator directive (2026-07-10): determine whether each panel has an actual
purpose defined by the specifications — wire the ones that do, remove the ones
that do not; then verify the console works 100% with good design.

Determination (evidence: `runtime-console`, `runtime-console-ux`,
`runtime-event-replay-entity-sync` specs + code map): **all six panels have a
spec-defined purpose AND a real backing concept already producing data on the
`agui.*` / REST side.** None meets the removal bar. The only gap is the
missing `runtime.*` emission arm plus thin frontend routing. So: wire all six,
remove none.

## What Changes

Backend (`src/uar/api/sse.rs`, `src/uar/domain/events.rs`, emitters):
- Add `runtime.memory_event` mapping in `to_runtime_entity_event` for
  `NormalizedEvent::MemoryMutation` / `MemoryRecall` (workflow-mirror + recall).
- Add `runtime.artifact` mapping for `NormalizedEvent::Artifact` /
  `ArtifactDisplay`.
- Enrich the `runtime.step` payload with `title` / `kind` / `summary` so
  Execution Timeline / Run Detail rows render non-blank (fixes an existing bug
  in an already-"wired" panel).
- Add a `NormalizedEvent::ProviderHealth` variant + emit `runtime.provider_health`
  from the existing `ProviderHealthMonitor` snapshot.
- Add a `NormalizedEvent::ModelRouteDecision` variant + emit
  `runtime.model_route_decision` (selected provider/model + reason) at
  route-decision time.
- Emit `runtime.a2ui_surface` for A2UI-typed surfaces/artifacts.

Frontend (`frontend/src/entities/runtime-ingest.ts`,
`frontend/src/stores/chat-stream-store.ts`, `runtime-console-page.tsx`):
- Route `agui.*` frames to `ingestAgUiEvent` (parallel to the existing
  `runtime.` guard) so the AG-UI Events panel populates.
- Confirm `EVENT_TYPE_MAP` + ingest handle the new `runtime.*` types.
- Remove the `NotWiredRuntimeState` banners now that every panel has live
  emission; retain honest empty states for genuinely-quiet panels.

Verification:
- UI/UX routing pass (impeccable audit/critique + design consult) on the
  console; polish for good design.
- Browser-drive every panel with real/replayed data as proof.
- `cargo test --lib` + frontend build/typecheck + runtime-ingest/e2e green.

## Capabilities

### Modified Capabilities
- `runtime-console-ux`: panels formerly disclosed as "not yet wired" are now
  wired to live backend emission; honest-disclosure requirement narrows to
  genuinely-quiet empty states only.

## Impact

Backend event model gains 2 `NormalizedEvent` variants + 3 new `runtime.*`
mappings + 1 payload enrichment; thin frontend routing; no panel removed.
KBD: change 5/9.
