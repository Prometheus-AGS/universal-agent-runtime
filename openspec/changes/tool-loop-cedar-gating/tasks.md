# Tasks — tool-loop-cedar-gating

## 0. Bootstrap
- [ ] 0.1 Confirm seams: gate closure (`manager.rs:912-918`), `tool_requires_approval` (`:258`), `is_tool_allowed` (`engine.rs:141`), engine construction (`server.rs:451`) + RunManager build (`~409`)
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. RunManager: optional governance engine
- [ ] 1.1 Add field `governance_engine: Option<Arc<GovernanceEngine>>` to `RunManager` (+ init `None` in constructor)
- [ ] 1.2 Add `#[must_use] pub fn with_governance_engine(mut self, engine: Arc<GovernanceEngine>) -> Self`

## 2. server.rs: construct engine before manager + wire
- [ ] 2.1 Move `GovernanceEngine::load_from_dir("policies")` construction to before the `RunManager::new(...)` builder chain
- [ ] 2.2 Add `.with_governance_engine(Arc::clone(&governance_engine))` to the builder chain

## 3. Gate closure: consult Cedar (deny → approval)
- [ ] 3.1 Capture `approval_agent_id = artifact.id.clone()` and a clone of the engine into the gate closure
- [ ] 3.2 At `:918`, compute `needs_approval = tool_requires_approval(&tool_name) || (engine && !is_tool_allowed(agent_id, tool_name).await)`; `Approved` when not needed
- [ ] 3.3 Use a policy-oriented `risk_reason` when approval is triggered by a Cedar deny

## 4. Validation (gate)
- [ ] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 4.2 `cargo clippy` — no new warnings in touched files
- [ ] 4.3 `cargo test --features postgres-backend --lib` — existing pass (+ any unit on the decision helper if extracted)
- [ ] 4.4 Manual: restrictive `forbid` policy → tool prompts for approval; permit-all default → unchanged; engine `None` → heuristic-only (pending live env — document)
- [ ] 4.5 `openspec validate tool-loop-cedar-gating --strict`; update `.kbd-orchestrator` progress

## Notes
- Permit-all `default.cedar` ⇒ no behavior change out of the box; heuristic kept as fallback (Rule 32).
- Deny → HITL approval (D2), not hard-reject. Engine optional (None ⇒ today's behavior).
- Spec delta overlaps `add-run-cancellation`'s MODIFIED of the same requirement (both additive) — reconcile at archive if both archived.
- Out of scope: removing the heuristic; hard-reject mode; user-scoped principal; policy UI.
