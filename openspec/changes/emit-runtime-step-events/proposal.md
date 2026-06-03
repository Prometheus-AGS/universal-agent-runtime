# emit-runtime-step-events

## Why

Closes carry-over goal **H3** from `uar-harness-parity` (planned as HP3, never built). The Runtime Console has no per-step run progress: the orchestrator runs a tool loop (`orchestrator.rs:334`, `iteration` counter at `:348`, cap `MAX_TOOL_ITERATIONS`) but emits no step signal, and there is no `RuntimeStep` event anywhere in `src/`. This is a **backend-only gap** — the frontend already ingests step events: `frontend/src/entities/runtime-ingest.ts:9-11` maps `step_started`/`step_updated`/`step_finished` → `RuntimeRunStep`. Closing H3 means emitting the events the console already waits for, through the existing `RunEventEmitter` + C4 `runtime.*` bus. **Not** the parking-lot `HookBus` (rejected in the parity assessment as redundant).

## What Changes

- **Base event:** add `crate::normalized::NormalizedEvent::RuntimeStep { step: u32, kind: RuntimeStepKind }` (`RuntimeStepKind` = `Started` | `Finished`). The orchestrator yields `Started` at the top of each tool-loop iteration (`orchestrator.rs:348`, after `iteration += 1`) and `Finished` at the end of that iteration's work.
- **Domain event:** add `crate::uar::domain::events::NormalizedEvent::RuntimeStep { run_id, step: u32, kind: String }`. The manager's base→domain consumption `match` maps the base marker, injecting `run_id`, and emits it via the existing `RunEventEmitter` (broadcast + 512-event replay).
- **Runtime bus:** add a `RuntimeStep` arm to `to_runtime_entity_event` (`sse.rs:388`) producing `("runtime.step", { type: "step_started"|"step_finished", id, run_id, step, updated_at })` — the shape `runtime-ingest.ts` already maps to `RuntimeRunStep`.

Out of scope (deferred): step emission on the graph-execution path (separate executor); a dedicated Console "steps" display panel (ingest is wired — display is separate UI/UX work); an `agui.step` event for chat clients (the Console uses `runtime.*`; add later only if a chat client needs it).

## Capabilities

### New Capabilities
- **`runtime-step-events`** — `specs/runtime-step-events/spec.md`. The runtime emits a started/finished step event per orchestrator iteration, carrying the run id and a monotonic step index, delivered on the `runtime.*` entity bus in the shape the Runtime Console ingests. Additive and non-breaking.

## Impact

- **Affected code:** `src/normalized.rs` (base `RuntimeStep` + `RuntimeStepKind`), `src/llm/orchestrator.rs` (yield Started/Finished per iteration), `src/uar/domain/events.rs` (domain `RuntimeStep`), `src/uar/runtime/manager.rs` (base→domain mapping + emit), `src/uar/api/sse.rs` (`to_runtime_entity_event` `runtime.step` arm). **No frontend change** (ingest already supports it); **no new dependency**.
- **APIs:** no HTTP changes; one new `runtime.*` entity event (`runtime.step`). Optional/none on the `agui.*` surface.
- **Provider compatibility:** unaffected — steps are loop-structural, provider-agnostic.
- **Realtime state:** the Runtime Console (Cockpit/Runs) gains per-iteration step progress via the existing emit/replay path; late-joiners get steps from the 512-event buffer.
- **Behavior preservation:** purely additive events; clients that ignore `runtime.step` are unaffected; no existing event changes.
- **Coverage:** the standard tool-loop path; the graph-execution path is a documented follow-up.
- **KBD workflow state:** YES — this phase closes carry-over H3.
