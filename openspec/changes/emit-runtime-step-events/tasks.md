# Tasks — emit-runtime-step-events

## 0. Bootstrap

- [x] 0.1 Confirm current seams: orchestrator loop/iteration (`orchestrator.rs:334/348`), two event models (`normalized.rs`, `uar/domain/events.rs`), manager base→domain `match` + `RunEventEmitter`, `to_runtime_entity_event` (`sse.rs:388`), frontend ingest (`runtime-ingest.ts:9-11`)
- [x] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Base event (normalized.rs + orchestrator)

- [x] 1.1 Add `enum RuntimeStepKind { Started, Finished }` and `NormalizedEvent::RuntimeStep { step: u32, kind: RuntimeStepKind }` to `src/normalized.rs`
- [x] 1.2 In `orchestrator.rs`, yield `RuntimeStep { step: iteration, kind: Started }` right after `iteration += 1` (`:348`)
- [x] 1.3 Yield `RuntimeStep { step: iteration, kind: Finished }` at the end of the iteration's work (once per iteration; guard against double-finish across early-return/break paths)
- [x] 1.4 Handle the new variant in any exhaustive base-event `match` (compiler-driven)

## 2. Domain event + manager mapping

- [x] 2.1 Add `NormalizedEvent::RuntimeStep { run_id, step: u32, kind: String }` to `src/uar/domain/events.rs`
- [x] 2.2 In the manager consumption `match`, map base `RuntimeStep` → domain (stringify kind `started`/`finished`, inject `run_id`) and emit via `RunEventEmitter`
- [x] 2.3 Handle the new domain variant in exhaustive matches (compiler-driven)

## 3. Runtime entity bus mapping (sse.rs)

- [x] 3.1 Add a `RuntimeStep` arm to `to_runtime_entity_event` → `("runtime.step", { type: "step_started"|"step_finished", id: "{run_id}-{step}", run_id, step, updated_at })`
- [x] 3.2 Ensure `to_agui_event` handles the domain `RuntimeStep` variant (return `None` — Console uses `runtime.*`; keeps the match exhaustive)

## 4. Tests + validation (gate)

- [x] 4.1 Unit test `to_runtime_entity_event(RuntimeStep{Started})` → `runtime.step` / `step_started`; and `Finished` → `step_finished`, with correct `id`/`run_id`/`step`
- [x] 4.2 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 4.3 `cargo clippy` — no new warnings in touched files
- [x] 4.4 `cargo test --features postgres-backend --lib` — existing pass + new mapping test
- [ ] 4.5 Manual: run with the Runtime Console open → step entities appear per iteration (pending live env — document if not runnable here)
- [x] 4.6 `openspec validate emit-runtime-step-events --strict`; update `.kbd-orchestrator` progress

## Notes

- **Option 1 (orchestrator base marker)** per decision D-A — touches both event models for true per-iteration fidelity.
- **started + finished** per decision D-B.
- Backend-only; frontend ingest already maps `RuntimeRunStep`. No new dependency.
- Out of scope: graph-path steps; `agui.step` for chat clients; a Console steps display panel.
- If `Finished` placement proves awkward across early-return paths, ship `started` first and add `finished` as a follow-up (design Open Questions).
