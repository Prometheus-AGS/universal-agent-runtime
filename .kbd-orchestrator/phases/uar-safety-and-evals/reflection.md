# Reflection: uar-safety-and-evals

**Phase:** `uar-safety-and-evals`
**Date:** 2026-06-03 · Backend: OpenSpec · Merged `main`: `4b26c1c`

## Executive summary

3 of 4 goals **MET** (S2/S3/S4 shipped + merged as PRs #31–#33); **S1 (eval harness) deliberately scoped out to its own future phase** (`uar-eval-harness`) at planning (decision D1) — it is greenfield and phase-sized. Merged `main` builds clean (`cargo check --features postgres-backend`), **235 lib tests pass**, all changes default-safe (no behavior change out of the box).

## Goal achievement

| Goal | Change | PR | Status |
|---|---|---|---|
| **S4** Tool-loop Cedar gating | tool-loop-cedar-gating | #31 | ✅ **MET** — `is_tool_allowed` consulted at the approval gate; deny → HITL approval; heuristic kept as fallback; permit-all default preserves behavior |
| **S3** Guardrail PII-block | guardrail-pii-block | #32 | ✅ **MET** — opt-in `block_on_pii` (default false) mirroring injection block; category-specific error code |
| **S2** Sycophancy auto-correction | sycophancy-auto-correct | #33 | ✅ **MET** — opt-in `auto_correct`; one corrective pass on a flagged turn → `SycophancyCorrected` follow-up; no first-response latency |
| **S1** Eval harness | — | — | ⏸️ **DEFERRED (by design, D1)** → dedicated `uar-eval-harness` phase |

Score: **3 MET / 1 deferred-by-design.**

## Delivered (merged)

| Change | PR | Surface |
|---|---|---|
| SE4 tool-loop-cedar-gating | #31 | `RunManager` optional `governance_engine` + builder; server reorder; gate closure = heuristic OR Cedar-deny |
| SE3 guardrail-pii-block | #32 | `block_on_pii` config; widened block gate; category-specific codes |
| SE2 sycophancy-auto-correct | #33 | `auto_correct` config; `quality::correction_messages`; `SycophancyCorrected` event + SSE; post-stream corrective pass |

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes shipped / planned (this phase) | 3 / 3 |
| Building clean on merged `main` | yes |
| New unit tests added | 2 (guardrail PII test fix + `correction_messages`) — net lib tests 234 → 235 |
| New compiler/clippy warnings | 0 (each introduced one fixed inline: missing config field in a test, `items_after_statements`) |
| `openspec validate --strict` | pass on all 3 |

No `artifact-refiner` logs (inline QA). No unrelated `cargo fmt` drift this phase (PR #28's earlier fix held).

## Decisions (resolved at plan, honored)

- **D1:** eval harness → own phase (kept this phase bounded).
- **D2:** Cedar deny → HITL approval (non-breaking; heuristic fallback retained).
- **D3:** sycophancy auto-correct opt-in, follow-up correction (no first-stream latency).
- **D4:** `block_on_pii` default false (behavior-preserving).

## Technical debt / deferrals (carried)

1. **S1 eval harness** — the genuine frontier; greenfield (`src/testing/` is dead CI analytics, not eval). Own phase `uar-eval-harness`.
2. **Live-env verification** pending across all three (policy-deny → approval prompt, PII block, corrective follow-up) — none runnable headlessly.
3. **Sycophancy correction** uses the app orchestrator model (not the per-run model); `reflect_threshold`/reflect-phase correction still unused; no thread-history rewrite (emits a follow-up event).
4. **MCP heartbeat loop** (status is connect-time only) and **config liveness** for in-flight runs remain prior carry-overs.
5. **Cedar tool gating uses `agent_id`** as principal; user-scoped policy is a follow-up.

## Lessons

- **Default-safe + opt-in is the throughline.** All three safety changes ship inert (permit-all policy, `block_on_pii=false`, `auto_correct=false`) — capability without behavior risk (Rule 32). Operators turn them on deliberately.
- **Re-grounding beat the old "defer" calls.** S3/S4 had clean single seams (the approval-gate closure; the input block gate) once re-assessed against current code — bounded changes, not the big lifts the parity phase assumed.
- **Reuse existing primitives.** SE2 reused `state.orchestrator.chat_non_streaming` rather than building a driver in the post-stream task; SE4 reused the existing HITL approval gate for deny handling. Smaller, safer diffs.
- **Splitting S1 out was the right scope call.** Bundling a greenfield eval subsystem here would have dominated and diluted the bounded safety wins.

## Recommended next

- **`uar-eval-harness` phase (S1):** rule-based + LLM-as-judge scorers, golden/prompt suites, persisted scores + regression comparison, a CLI/endpoint runner. Likely multi-change; assess greenfield.
- Then the smaller carry-overs: **MCP heartbeat loop**, **live-env smoke harness**, **config liveness**, **user-scoped Cedar principal**, **durable workflows** (own phase).

## Status
3 MET / 1 deferred-by-design. All 3 shipped changes merged (#31–#33); merged `main` builds + 235 tests pass. Ready for `/kbd-new-phase` (recommend `uar-eval-harness`).
