## Context

Two event models: the orchestrator emits `crate::normalized::NormalizedEvent` (base), consumed by the manager's spawned task which re-emits `crate::uar::domain::events::NormalizedEvent` (domain, with `run_id`) via `RunEventEmitter` (broadcast + 512-event replay). The orchestrator tool loop (`orchestrator.rs:334`) has an `iteration` counter (`:348`, after `iteration += 1`). C4's `to_runtime_entity_event` (`sse.rs:388`) maps domain events → `runtime.*` entity events, invoked per event at `server.rs:4056`. The frontend `runtime-ingest.ts:9-11` already maps `step_started`/`step_updated`/`step_finished` → `RuntimeRunStep`. No `RuntimeStep` exists in either model today.

## Goals / Non-Goals

**Goals:** emit a faithful per-iteration step (`started`+`finished`) carrying run id + monotonic index, delivered as `runtime.step` in the shape the Console already ingests, via the existing emitter. Additive, non-breaking, no new dependency.

**Non-Goals:** graph-execution-path steps; an `agui.step` event (Console uses `runtime.*`); a Console steps *display* panel (ingest only); sub-step granularity (one step = one tool-loop iteration).

## Decisions

### D1 — Option 1: orchestrator yields a base `RuntimeStep` marker (per decision D-A)
Add `crate::normalized::NormalizedEvent::RuntimeStep { step: u32, kind: RuntimeStepKind }` with `enum RuntimeStepKind { Started, Finished }`. The orchestrator yields `RuntimeStep { step: iteration, kind: Started }` right after `iteration += 1` (`orchestrator.rs:348`) and `RuntimeStep { step: iteration, kind: Finished }` at the end of that iteration's body (before looping for the next turn / on the paths that conclude the iteration).
- **Why over manager-side milestones (Option 2):** the iteration counter is the true per-LLM-turn boundary; the manager only sees deltas/tool events and can't reconstruct iteration boundaries cleanly. Cost: touches both event models — accepted for fidelity.
- **`Finished` placement:** emit once per iteration on the path(s) that end an iteration. Keep it simple — emit `Finished` immediately before the loop continues to the next iteration and before the terminal `Done`/break, guarding against a double-finish for the same index.

### D2 — Domain mapping in the manager (per decision D-B: started+finished)
Add `uar::domain::events::NormalizedEvent::RuntimeStep { run_id, step: u32, kind: String }` (`kind` = `"started"|"finished"`). In the manager's base→domain `match`, map the base `RuntimeStep`, stringify the kind, inject `run_id`, and emit via `RunEventEmitter` like other domain events (so it lands in broadcast + replay).
- **Why `String` kind in the domain event:** the domain/SSE layer already uses string discriminators; keeps serialization simple and matches sibling events.

### D3 — `runtime.step` entity mapping
Add a `RuntimeStep` arm to `to_runtime_entity_event`:
`("runtime.step", { type: format!("step_{kind}"), id: format!("{run_id}-{step}"), run_id, step, updated_at: now })`. The composite `id` keeps each step a distinct `RuntimeRunStep` entity; `type` becomes `step_started`/`step_finished` exactly as `runtime-ingest.ts` expects.

### D4 — Optional `agui.step` — omitted for now
Skip a `to_agui_event` arm; the Console consumes `runtime.*`. (Returning `None` from `to_agui_event` for `RuntimeStep` is fine — the match must still handle the new variant exhaustively.) Add later only if a chat client needs steps.

## Risks / Trade-offs

- **[Touches both event models]** Option 1 adds a variant to base + domain enums; every exhaustive `match` on them must handle it → Mitigation: compiler enforces exhaustiveness; `to_agui_event` returns `None` for it; small, mechanical.
- **[Double / missing finish]** mis-placing `Finished` could double-count or skip → Mitigation: emit `Started` once at loop top, `Finished` once per iteration end; keep indices tied to the single `iteration` counter; unit/behavior check on a multi-iteration path.
- **[Replay buffer pressure]** 2 extra events/iteration (≤ `MAX_TOOL_ITERATIONS`=10 ⇒ ≤20 events) is negligible against the 512-event buffer.
- **[Cardinality/UX]** steps are per-run entities keyed by `{run_id}-{step}` — bounded; no metric-label cardinality concern (not a metric).

## Migration Plan

1. Add base `RuntimeStep` + `RuntimeStepKind` to `normalized.rs`.
2. Yield Started/Finished in the orchestrator loop.
3. Add domain `RuntimeStep`; map base→domain in the manager; emit.
4. Add the `runtime.step` arm to `to_runtime_entity_event`; ensure `to_agui_event` handles the variant (returns `None`).
5. Unit-test the `to_runtime_entity_event` mapping (started + finished); `cargo check`/`clippy`/lib tests green.
- **Rollback:** additive; revert restores the prior (no-step) state. No data migration.

## Open Questions

- **`Finished` semantics:** does the Console need `finished` to compute duration, or is `started` sufficient for the v1 panel? (We emit both per D-B; if `finished` placement proves awkward across early-return paths, ship `started` first and add `finished` in a follow-up.)
- **Graph path:** emit steps for graph-node execution too? (Deferred — separate executor, its own iteration concept.)
