# tool-loop-cedar-gating

## Why

HP6 mounted the Cedar `governance_layer` at the HTTP layer, but the **orchestrator tool loop still authorizes tools solely via the 6-keyword `tool_requires_approval` heuristic** (`manager.rs:258`, consulted in the approval-gate closure at `manager.rs:918`). `GovernanceEngine::is_tool_allowed(agent_id, tool_name)` exists and the `execute_tool` Cedar action + `tool-approval.cedar` policy are present, but the engine is **never consulted at tool execution**. This is goal **S4** of `uar-safety-and-evals`: make Cedar policy actually gate tool calls.

## What Changes

- **Thread the `GovernanceEngine` into `RunManager`** (it currently lives only in `AppState`/the HTTP layer): add an optional `governance_engine: Option<Arc<GovernanceEngine>>` field + a `with_governance_engine(...)` builder, set in `server.rs` where the engine is constructed (the engine's construction is reordered before `RunManager` so it can be passed in).
- **Consult Cedar in the approval-gate closure** (`manager.rs:918`): a tool requires approval when the keyword heuristic flags it **OR** the governance engine denies it. Concretely, replace `if !tool_requires_approval(&tool_name) { return Approved }` with a check that also calls `is_tool_allowed(agent_id, tool_name)` (when an engine is configured) and treats a **deny as requires-approval** (decision D2: deny → HITL approval gate, so a human can approve; the heuristic stays as a fallback trigger).
- **Capture the run's `agent_id`** (`artifact.id`) into the gate closure so the Cedar request is built for the right principal.

Default behavior is unchanged: the bundled `default.cedar` is permit-all, so `is_tool_allowed` returns true for every tool until an operator adds restrictive policies; combined with the unchanged heuristic, the approval flow is identical out of the box.

Out of scope (deferred): removing/replacing the keyword heuristic entirely (kept as fallback per D2); a deny→hard-reject mode (we route to approval); policy authoring/UI; per-tool risk metadata.

## Capabilities

### Modified Capabilities
- **`tool-approval-workflow`** — delta `specs/tool-approval-workflow/spec.md`. The existing "Tool calls matching approval policy pause for user confirmation" requirement is extended: a tool also pauses for approval when Cedar policy denies it (not only when the keyword heuristic matches). Existing approve/reject/timeout/cancel behavior is unchanged.

## Impact

- **Affected code:** `src/uar/runtime/manager.rs` (new optional `governance_engine` field + `with_governance_engine` builder; capture `agent_id`; extend the gate decision at `:918`), `src/server.rs` (construct the `GovernanceEngine` before `RunManager` and pass it via the builder). No new dependency.
- **APIs:** none changed. Behavior change only when restrictive Cedar policies exist (permit-all default preserves current behavior).
- **Runtime/UX:** a policy-denied tool now surfaces the existing approval prompt (HITL) instead of executing silently; the user can approve/reject as today.
- **Security (Rule 33):** closes the gap where HTTP-layer policy did not protect tool execution; tool calls are now policy-checked at the loop. Deny→approval keeps a human in control rather than silently blocking.
- **Behavior preservation:** permit-all default + heuristic-OR-deny semantics ⇒ no change unless policies are authored. Engine is optional (`None` ⇒ heuristic-only, exactly as today).
- **KBD workflow state:** YES — S4 of `uar-safety-and-evals`.
