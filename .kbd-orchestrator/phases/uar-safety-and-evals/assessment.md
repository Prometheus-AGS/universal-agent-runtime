# Assessment: uar-safety-and-evals

**Phase:** `uar-safety-and-evals`
**Date:** 2026-06-03 · Backend: OpenSpec · base `main` `6583087`
**Origin:** deferred frontier from `uar-harness-parity` (H7 + safety follow-ups)

## Goals (this phase)

| # | Goal | Origin |
|---|---|---|
| S1 | **Eval harness** — rule-based + LLM-as-judge scorers, prompt/golden suites, persisted regression metrics over time | HP7 (deferred at parity planning, R3) |
| S2 | **Sycophancy auto-correction** — act on a flagged response (regenerate/correct), honoring `auto_correct_threshold`/`log_only`/`reflect_threshold` | HP4 follow-up |
| S3 | **Guardrail hardening** — PII-block mode + stronger injection handling beyond the current detect/injection-block | HP6 follow-up |
| S4 | **Tool-loop Cedar gating** — replace the 6-keyword `tool_requires_approval` heuristic with `is_tool_allowed` at the orchestrator tool loop | HP6 deferred |

## Current state (grounded)

- **S1 Eval harness — none.** `src/testing/` exists (alerting/analytics/monitoring/performance/reliability/…) but is **dead code: `mod testing` is not declared in `lib.rs`/`main.rs`**, and it is CI-flakiness analytics, not LLM eval. No scorers, prompt suites, golden sets, faithfulness/toxicity/relevance scoring, or persisted regression metrics anywhere in compiled `src/`. **Greenfield.**
- **S2 Sycophancy — detection only.** `quality::detect` (`src/uar/quality.rs:62`) returns `SycophancyOutcome { flagged, ... }`; the server emits `SycophancyFlagged` post-stream. `auto_correct_threshold` is used only as the flag threshold; **`reflect_threshold` and `log_only` are unused** (inert config); no correction/regeneration occurs.
- **S3 Guardrails — injection-block only.** `server.rs:3730` blocks when `block_on_injection` AND `finding.category == Injection`; **PII is always flag-only** (no block mode). Injection detection is the heuristic phrase list in `guardrails.rs`.
- **S4 Tool approval — still heuristic.** `tool_requires_approval` (`manager.rs:258`, 6 keywords) is consulted at the tool loop (`manager.rs:918`); `GovernanceEngine::is_tool_allowed` exists and the Cedar layer is mounted at HTTP (HP6) but **is never called at the orchestrator tool loop**. The Cedar `execute_tool` action + `tool-approval.cedar` policy already exist.

## Complexity & risk per goal

- **S1 Eval harness: LARGE / Medium-High risk.** A new subsystem — scorers (rule + LLM-judge), a run/prompt corpus, persistence for scores, a runner + regression comparison, and a surface (CLI/endpoint). LLM-judge needs a model + cost/determinism care. This goal alone is phase-sized.
- **S2 Sycophancy auto-correction: MEDIUM.** Requires a second LLM pass (regenerate with a correction prompt) on the chat path, careful UX (don't double-stream), and honoring thresholds. Touches the orchestrator/manager response path.
- **S3 PII-block + injection hardening: SMALL-MEDIUM.** Add a `block_on_pii` config + block path (mirror injection); optionally expand heuristics. Bounded.
- **S4 Tool-loop Cedar gating: MEDIUM.** Call `is_tool_allowed(agent_id, tool_name)` at `manager.rs:918`, mapping deny → the existing approval/deny flow; keep the heuristic as a fallback or migrate it into policy. Behavior-sensitive (could block tools) — needs permit-safe defaults.

## Recommended decomposition (feeds `/kbd-plan`)

Order by leverage + risk-containment; **S1 is large enough to consider splitting**:

1. **SE4 — `tool-loop-cedar-gating` (S4)** — highest-value safety, bounded, builds directly on HP6's mounted engine. Permit-safe by default (permit-all policy unchanged).
2. **SE3 — `guardrail-pii-block` (S3)** — small; add `block_on_pii` + block path; mirror the injection-block pattern.
3. **SE2 — `sycophancy-auto-correct` (S2)** — medium; regenerate on flag (opt-in via `log_only=false` semantics), honor thresholds.
4. **SE1 — `eval-harness` (S1)** — LARGE. **Recommend scoping to a v1**: rule-based scorers + a golden prompt suite + persisted scores + a CLI/endpoint runner; defer LLM-as-judge + regression dashboards to a follow-up. *Or* split S1 into its own phase `uar-eval-harness` and keep this phase to S2-S4.

## Key product decisions (for `/kbd-plan`)

- **D1 — Eval harness scope/placement:** v1-in-this-phase (rule-based + golden + persist, judge deferred) vs. its own dedicated phase. (S1 is phase-sized; recommend its own phase, or a tightly-scoped v1.)
- **D2 — Tool-loop Cedar semantics:** does `is_tool_allowed` deny → hard-reject, or deny → route to the HITL approval gate? And is the keyword heuristic kept as a fallback or fully replaced by policy?
- **D3 — Sycophancy auto-correct UX:** correct silently before first token (adds latency) vs. emit a correction as a follow-up message vs. flag-and-offer. And default on/off.
- **D4 — PII-block default:** keep `block_on_pii` default false (preserve behavior) — confirm.

## Assessment status

- Four goals scoped; current state grounded (eval harness greenfield; S2-S4 bounded follow-ups with known seams).
- Decomposition proposed (SE4 → SE3 → SE2 → SE1), with S1 flagged as phase-sized.
- Product decisions D1-D4 surfaced.
- Ready for `/kbd-plan uar-safety-and-evals` (resolve D1-D4 first).
