PLAN: emit-runtime-step-events
Project: universal-agent-runtime
Date: 2026-06-03
OpenSpec available: YES
Planning model: Opus 4.8 (frontier) — satisfies kbd-plan model policy
Changes to implement: 1

---

## Decisions resolved (this plan is grounded in them)

- **D-A signal origin → Option 1 (orchestrator base marker):** the orchestrator yields a base `RuntimeStep` at each tool-loop iteration; the manager maps base→domain `RuntimeStep`. Most faithful (true per-LLM-iteration boundary).
- **D-B granularity → started + finished:** emit `step_started` at iteration top and `step_finished` at iteration end (console gets full lifecycle + duration).
- **D-C coverage → tool-loop path now; graph-execution path is a documented follow-up.**

Single change closes carry-over goal **H3**. Backend-only — the frontend (`runtime-ingest.ts`) already maps `step_started/updated/finished` → `RuntimeRunStep`.

---

## CHANGE LIST (ordered)

1. **emit-runtime-step-events**: emit per-orchestrator-iteration step lifecycle events through the existing `RunEventEmitter` → C4 `runtime.*` bus, so the Runtime Console shows step progress.
   - Scope: orchestrator (base event) | event models (base + domain) | manager (mapping/emit) | sse (`runtime.step` arm)
   - Depends on: NONE (base `main`; prior phase merged)
   - Recommended agent: Claude Code
   - Est. complexity: S–M
   - Complexity score: Low–Medium
   - Model class: medium
   - Customer value: MEDIUM (observability — live step progress in the Runtime Console)
   - Details:
     - **Base event:** add `crate::normalized::NormalizedEvent::RuntimeStep { step: u32, kind: RuntimeStepKind }` (kind = `Started` | `Finished`). Yield `Started` at the orchestrator loop top (`orchestrator.rs:348`, after `iteration += 1`) and `Finished` at iteration end (before the loop continues / on break to next turn).
     - **Domain event:** add `uar::domain::events::NormalizedEvent::RuntimeStep { run_id, step: u32, kind: String }`; map base→domain in the manager consumption `match` (inject `run_id`).
     - **Runtime bus:** add a `RuntimeStep` arm to `to_runtime_entity_event` (`sse.rs:388`) → `("runtime.step", { type: "step_started"|"step_finished", id: "<run_id>-<step>", run_id, step, updated_at })` matching the `RuntimeRunStep` shape `runtime-ingest.ts` expects.
     - **(Optional, low priority)** `to_agui_event` `agui.step` arm for chat clients — include only if trivial; Console uses `runtime.*`.
     - **Tests:** unit-test the `to_runtime_entity_event` `RuntimeStep` mapping (started + finished); confirm `cargo check`/`clippy` clean + lib tests pass.
     - **Out of scope:** graph-execution-path steps (follow-up); a dedicated Console "steps" display panel (ingest is wired; display is separate UI/UX work).

---

## EXECUTION ROUND ORDER

- **Round 1:** `emit-runtime-step-events` (single change, no dependencies).

---

## COMMANDS TO RUN

```
/opsx:new emit-runtime-step-events
```

---

## Sycophancy self-check

- **S-02 (grounding):** every integration point is file:line-cited from the assessment against current `main`; the frontend-already-supports-steps claim is verified (`runtime-ingest.ts:9-11`).
- **S-07 (scope creep):** held to one additive change; graph-path steps, an `agui.step` event, and a Console display panel are explicitly out of scope.
- **S-03 (caveat):** trade-offs surfaced — Option 1 touches both event models (chosen for fidelity over Option 2's smaller footprint); coverage limited to the tool-loop path; display polish deferred.

PLAN COMPLETE
