## Context

`RunManager` builds a per-run `ToolApprovalGate` closure (`manager.rs:912`); at `:918` it returns `Approved` immediately unless `tool_requires_approval(&tool_name)` (the 6-keyword heuristic, `:258`) matches, in which case it emits `ToolCallApprovalRequired` and awaits the HITL `oneshot`. `GovernanceEngine::is_tool_allowed(agent_id, tool_name) -> bool` (`engine.rs:141`, async, builds an `execute_tool` Cedar request, denies on error) exists; the engine is constructed in `server.rs:451` and lives in `AppState` (used by the HTTP `governance_layer`, HP6) but is NOT held by `RunManager`. The bundled `policies/default.cedar` is permit-all.

## Goals / Non-Goals

**Goals:** consult Cedar at the tool loop so policy-denied tools pause for HITL approval; keep the heuristic as a fallback; preserve behavior under the permit-all default; make the engine optional on `RunManager`.
**Non-Goals:** removing the heuristic; a deny→hard-reject mode (route to approval per D2); policy authoring/UI; per-tool risk metadata; gating non-tool actions (call_llm etc.).

## Decisions

### D1 — Thread the engine into RunManager (optional)
Add `governance_engine: Option<Arc<GovernanceEngine>>` to `RunManager` + `#[must_use] pub fn with_governance_engine(mut self, e: Arc<GovernanceEngine>) -> Self`. In `server.rs`, move the `GovernanceEngine::load_from_dir("policies")` construction (currently ~451) to BEFORE the `RunManager::new(...)` builder chain (~409) and add `.with_governance_engine(Arc::clone(&governance_engine))`. `None` ⇒ heuristic-only (exactly today's behavior); keeps the manager testable without an engine.
- **Why optional:** the manager is constructed in contexts/tests without an engine; optionality preserves them and makes the change additive.

### D2 — Gate decision = heuristic OR policy-deny (deny → approval)
At `manager.rs:918`, compute `needs_approval = tool_requires_approval(&tool_name) || policy_denies`, where `policy_denies = match &governance { Some(e) => !e.is_tool_allowed(&agent_id, &tool_name).await, None => false }`. If `!needs_approval` → `Approved` (unchanged fast path). Otherwise fall through to the existing approval-required flow (emit event, await oneshot, timeout/reject). A policy deny thus becomes a HITL prompt (D2), with the heuristic still triggering independently.
- **Why OR (not replace):** keeps the heuristic as a safety fallback (Rule 32); permit-all default ⇒ `policy_denies` is always false ⇒ identical behavior out of the box.
- **risk_reason:** when approval is triggered by policy (not heuristic), use a policy-oriented reason string so the UI explains why.

### D3 — Capture agent_id into the closure
The gate closure currently captures `approval_run_id`, `approval_emitter`, `approval_pending`. Add `approval_agent_id = artifact.id.clone()` and a clone of the engine (`Option<Arc<GovernanceEngine>>`) into the closure so the per-call async block can build the Cedar request for the correct principal.

## Risks / Trade-offs

- **[Deny storms / latency]** an `is_tool_allowed` call per tool invocation adds an async Cedar eval → Mitigation: Cedar eval is in-memory and fast; only runs per tool call; engine optional.
- **[Behavior change surprise]** mounting policy at the loop could pause tools an operator didn't expect → Mitigation: permit-all default ⇒ no change until policies authored; deny→approval (not reject) keeps a human in control; documented.
- **[Construction ordering]** moving engine construction before the manager in server.rs → Mitigation: engine load depends only on the `policies/` dir; no dependency on the manager; straightforward reorder.
- **[Spec delta overlap]** `add-run-cancellation` also MODIFIED this requirement (cancel-while-awaiting scenario) → Mitigation: both extend the same requirement additively; reconcile at archive time if both are archived (note in tasks).

## Migration Plan
1. Add the optional field + builder to `RunManager`.
2. Reorder `server.rs` to build the engine before the manager; wire `.with_governance_engine`.
3. Capture `agent_id` + engine clone into the gate closure; extend the `:918` decision.
4. `cargo check`/`clippy`/tests; manual: with a restrictive `tool-approval.cedar` `forbid`, the tool prompts for approval; permit-all ⇒ unchanged.
- Rollback: additive (optional field, OR-condition); revert restores heuristic-only.

## Open Questions
- Should the run carry a distinct policy principal (e.g., user id) rather than `agent_id`? (Use `agent_id` now — matches `is_tool_allowed`'s contract; user-scoped policy is a follow-up.)
