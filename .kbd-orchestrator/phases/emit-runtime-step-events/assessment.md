# Assessment: emit-runtime-step-events

**Phase:** `emit-runtime-step-events` (closes carry-over goal **H3** from `uar-harness-parity`)
**Date:** 2026-06-03
**Project:** Universal Agent Runtime
**Backend:** OpenSpec · base `main` `db02703`
**Assessed by:** kbd-assess (direct, file:line-grounded)

---

## Goal

Emit per-orchestrator-iteration **RuntimeStep** events so the Runtime Console shows step-by-step run progress. This was planned as HP3 in the prior phase, cut-listed, but **never implemented** — it is the top carry-over. Original direction (still valid): add a `RuntimeStep` event, emit per orchestrator iteration, flow through the existing `RunEventEmitter` → SSE/Console. **Explicitly NOT** the parking-lot `HookBus` (killed in prior assessment as redundant).

---

## Headline finding: this is a **backend-only** gap — the frontend is already ready

`frontend/src/entities/runtime-ingest.ts:9-11` already maps:
```
step_started:  "RuntimeRunStep"
step_updated:  "RuntimeRunStep"
step_finished: "RuntimeRunStep"
```
So the Runtime Console's ingest layer **already understands a `runtime.step` entity event** (`RuntimeRunStep`). The backend simply never emits it. Closing H3 means producing the events the console is already waiting for — no frontend work required (a panel to *display* steps may be a separate UI follow-up, but ingest is wired).

---

## Current state (what exists)

1. **Two event models** (unchanged since prior phase):
   - **Base**: `crate::normalized::NormalizedEvent` — emitted by the orchestrator (`StreamStart`, `MessageDelta`, `ToolCallDelta`, `ToolResult`, `Done`, `Error`, …). No step variant.
   - **Domain**: `crate::uar::domain::events::NormalizedEvent` — carries `run_id`, emitted by the manager via `RunEventEmitter`; mapped to SSE. No `RuntimeStep` variant (`grep` confirms RuntimeStep absent everywhere in `src/`).

2. **The orchestrator already counts iterations.** `src/llm/orchestrator.rs:332` `let mut iteration = 0;`, `:334 loop {`, `:348 iteration += 1;` (cap `MAX_TOOL_ITERATIONS = 10`, `:66`). This is the natural per-iteration emission point — but it lives in the base event stream consumed by the manager.

3. **The runtime entity-event bus (C4) is the delivery mechanism.** `to_runtime_entity_event` (`src/uar/api/sse.rs:388`) maps domain events → `runtime.run` / `runtime.tool_call` / `runtime.approval`. It is invoked at `src/server.rs:4056` for each event, emitting `runtime.*` alongside `agui.*`. **There is no `runtime.step` arm** — adding one is the delivery half.

4. **The manager maps base→domain events** in its consumption loop (`manager.rs`, the big `match base_event { ... }`), and emits via `RunEventEmitter` (broadcast + 512-event replay). This is where a base step marker would become a domain `RuntimeStep`.

---

## The gap (precise, ordered)

1. **No domain `RuntimeStep` event** — add `NormalizedEvent::RuntimeStep { run_id, step: u32, kind: String }` (or a small enum kind: `started`/`finished`) to `uar/domain/events.rs`.
2. **No per-iteration signal from the orchestrator** — the orchestrator must surface "iteration N began". Two options (see Decisions).
3. **No `runtime.step` mapping** — add a `RuntimeStep` arm to `to_runtime_entity_event` (`sse.rs`) producing `("runtime.step", { type: "step_started"|"step_finished", id, run_id, step, … })` so the existing `RuntimeRunStep` ingest fires.
4. **(Optional) no `agui.*` step mapping** — add a `to_agui_event` arm (`agui.step`) if chat clients should also see steps. Lower priority; the Console uses `runtime.*`.

---

## Decisions to make (feed `/kbd-plan`)

### D-A: Where does the per-iteration signal originate?
- **Option 1 — Orchestrator yields a base `RuntimeStep` marker** at loop top (`orchestrator.rs:348`), manager maps base→domain `RuntimeStep`. *Pros:* accurate per-LLM-turn boundary; uses the real iteration counter. *Cons:* touches both event models (add a `crate::normalized::NormalizedEvent::RuntimeStep` variant + the orchestrator) and the manager's mapping. **Recommended** — it is the truest "step" and matches the original plan (`orchestrator.rs:348`).
- **Option 2 — Manager-side milestones only** — emit `RuntimeStep` from the manager at events it already sees (`RunStart` → step 0; each `ToolStart`/turn). *Pros:* no base-event/orchestrator change; smaller. *Cons:* coarser; "step" = tool boundary, not LLM iteration; less faithful to the loop.
- Recommend **Option 1** for fidelity; Option 2 is the low-touch fallback.

### D-B: Step granularity / `kind`
What does a "step" represent and what kinds to emit? Proposal: one step per orchestrator iteration with `step_started` at iteration top and `step_finished` at iteration end (or just `step_started` to start simple). Confirm whether `step_finished`/`step_updated` are needed now (the frontend supports all three) or just `step_started`.

### D-C: Emit on the chat path only, or all run paths?
The graph-execution branch (`manager.rs`) is a separate executor; steps there would need their own emission. Recommend: cover the standard tool-loop path now; note the graph path as follow-up (consistent with prior phase scoping).

---

## Integration points (current `main`)

| Concern | Location |
|---|---|
| Orchestrator loop + iteration counter | `src/llm/orchestrator.rs:332-348` |
| Base event model | `src/normalized.rs` (`NormalizedEvent`) |
| Domain event model | `src/uar/domain/events.rs` (`NormalizedEvent`) |
| Manager base→domain mapping + emit | `src/uar/runtime/manager.rs` (consumption `match`, `RunEventEmitter`) |
| Runtime entity-event mapping | `src/uar/api/sse.rs:388` (`to_runtime_entity_event`) |
| Runtime event emission seam | `src/server.rs:4056` |
| Frontend ingest (already supports step) | `frontend/src/entities/runtime-ingest.ts:9-11` (`RuntimeRunStep`) |

---

## Complexity & risk

- **Complexity:** Small–Medium (S–M). Additive: a new event variant in one/both models + one mapping arm + emission calls. No new dependency, no new infrastructure (reuses `RunEventEmitter` + C4 bus).
- **Risk:** Low. Additive events; `to_runtime_entity_event` returns `Some` only for the new arm; clients that ignore `runtime.step` are unaffected. Frontend ingest already handles the entity type.
- **Behavior preservation:** no change to existing events; steps are new signals.

---

## Proposed scope (single change)

One change `emit-runtime-step-events`:
- Add `RuntimeStep` to the domain event model (+ base model if Option 1).
- Emit per orchestrator iteration (Option 1) or per manager-visible milestone (Option 2).
- Map to `runtime.step` in `to_runtime_entity_event` (`step_started`/`step_finished`).
- Unit-test the mapping; verify the merged build + tests; manual Console check pending live env.

Out of scope: a dedicated Console "steps" *panel* (ingest is wired; display polish is separate UI work + UI/UX routing); graph-path step emission; `agui.step` for chat clients (optional).

---

## Assessment status

- Gap is backend-only and precisely located; frontend ingest already supports `RuntimeRunStep`.
- Decisions surfaced: D-A (signal origin — recommend Option 1), D-B (granularity/kind), D-C (path coverage).
- Ready for `/kbd-plan emit-runtime-step-events`.
