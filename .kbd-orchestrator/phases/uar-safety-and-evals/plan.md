PLAN: uar-safety-and-evals
Project: universal-agent-runtime · Date: 2026-06-03 · OpenSpec: YES
Planning model: Opus 4.8 (frontier)
Changes to implement: 3 (S1 eval harness deferred to its own phase)

---

## Decisions resolved

- **D1 — eval harness (S1):** **deferred to a dedicated future phase `uar-eval-harness`.** Not in this phase. This phase = the bounded safety follow-ups S2/S3/S4.
- **D2 — tool-loop Cedar deny:** a deny **routes to the existing HITL approval gate** (user can approve); the keyword heuristic stays as a fallback (non-breaking).
- **D3 — sycophancy auto-correct:** **opt-in; emit a corrected response as a follow-up** (never block/delay the first stream).
- **D4 — PII-block:** add `block_on_pii`, **default false** (detect-only preserved; opt-in).

Scope cut (S-07/S-03): the eval harness is explicitly OUT (own phase). All three changes are additive and default-safe.

---

## CHANGE LIST (ordered)

1. **tool-loop-cedar-gating** (S4): consult Cedar `is_tool_allowed` at the orchestrator tool loop; a deny routes to the HITL approval gate.
   - Scope: orchestrator/manager tool loop | governance
   - Depends on: NONE (builds on HP6's mounted engine + existing approval gate)
   - Agent: Claude Code · Complexity: M · Score: Medium · Model: medium · Value: HIGH
   - Details: At `manager.rs:918` (tool-approval decision), call `GovernanceEngine::is_tool_allowed(agent_id, tool_name)`. If **denied** → drive the existing approval gate (treat as requires-approval) so a human can approve; if allowed → proceed. Keep `tool_requires_approval` (the 6-keyword heuristic) as an additional trigger (a tool requires approval if the heuristic flags it OR Cedar denies it). Permit-all `default.cedar` ⇒ no behavior change until restrictive policies exist. Needs the engine + agent_id in scope at the loop (thread from the run).

2. **guardrail-pii-block** (S3): add an opt-in PII block mode mirroring the injection block.
   - Scope: config | chat input seam (server.rs)
   - Depends on: NONE
   - Agent: Claude Code · Complexity: S–M · Score: Low–Medium · Model: medium · Value: MEDIUM
   - Details: Add `GuardrailsConfig.block_on_pii: bool` (default false). At `server.rs:3730` extend the block condition: block when `(block_on_injection && Injection) || (block_on_pii && Pii)`. Reuse the existing guardrail error + `GuardrailFlagged` emit. Detect-only remains the default for both categories.

3. **sycophancy-auto-correct** (S2): opt-in follow-up correction on a flagged response.
   - Scope: config | quality | chat post-stream seam (server.rs) | orchestrator (correction pass)
   - Depends on: NONE (extends HP4's detection)
   - Agent: Claude Code · Complexity: M · Score: Medium · Model: frontier · Value: MEDIUM
   - Details: Add `SycophancyConfig.auto_correct: bool` (default false; `log_only`/`reflect_threshold` become meaningful). When `auto_correct` and a response is flagged at/above `auto_correct_threshold` (and not `log_only`), run ONE corrective LLM pass (a correction-prompt regeneration of the assistant text) AFTER the terminal event, and emit it as a follow-up message/event (do not delay or re-stream the original). Honor thresholds; bounded to one correction attempt; cost/latency contained to flagged turns only.

---

## EXECUTION ROUND ORDER

- **Round 1 (independent):** all three. Sequence **SE4 → SE3 → SE2** to land the smaller/safer safety changes first and keep the manager/orchestrator diffs clean before the more involved correction pass.

## DEFERRED (out of phase)

- **S1 eval harness → dedicated `uar-eval-harness` phase** (rule-based + LLM-judge scorers, golden suites, persisted regression metrics, runner). The dead `src/testing/` tree is unrelated CI analytics.
- MCP heartbeat loop; live-env smoke harness; config liveness for in-flight runs (carry-overs).

## COMMANDS TO RUN

```
/opsx:new tool-loop-cedar-gating
/opsx:new guardrail-pii-block
/opsx:new sycophancy-auto-correct
```

## Sycophancy self-check
- S-02: each change cites a concrete current seam (manager.rs:918, server.rs:3730, the post-stream sycophancy seam).
- S-07: eval harness cut to its own phase; all three changes default-off/permit-safe — no scope creep.
- S-03: trade-offs surfaced — D2 (deny→approval vs reject), D3 correction latency/UX, permit-all default keeping S4 non-breaking.

PLAN COMPLETE
