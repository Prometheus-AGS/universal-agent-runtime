# Reflection: uar-harness-parity

**Phase:** `uar-harness-parity`
**Date:** 2026-06-02
**Project:** Universal Agent Runtime
**Backend:** OpenSpec · **Source of truth:** `.kbd-orchestrator`
**Reflection discipline:** sycophancy-corrected — goal status reported honestly, including an unplanned gap.

---

## Executive summary

Of the phase's 8 goals, **5 MET, 1 PARTIAL, 2 NOT MET** (one deferred by design, one an **unintended execution gap**). Five of seven planned change-sets shipped and merged to `main` as PRs #23–#27; the merged tree builds clean (`cargo check --features postgres-backend`) with **232 lib tests passing** and zero new warnings. The runtime moved from "half-built observability + no cancellation + dead guardrails" to a runtime with real distributed tracing, run cancellation, response-quality + input guardrails, and resumable streaming.

**The honest miss:** **HP3 (`emit-runtime-step-events`, goal H3) was never implemented.** It was in the plan's suggested cut (C0+HP1–HP4) but was skipped during execution — the per-change selection went HP1 → HP2 → HP4 → HP5 → HP6, bypassing HP3. RuntimeStep (per-orchestrator-iteration progress) events do not exist. This is carry-over, not a deferral decision.

---

## Goal achievement

| Goal | Change | Status | Evidence |
|---|---|---|---|
| **H1** Run cancellation (disconnect/shutdown/explicit) | HP1 `add-run-cancellation` | ✅ **MET** | PR #23; per-run `CancellationToken`, `POST /runs/{id}/cancel`, last-subscriber-drop, shutdown root token, `Cancelled` event |
| **H2** OTLP tracing + per-LLM latency + cost | HP2 `wire-otlp-tracing-and-cost` | ✅ **MET** | PR #24; OTLP/HTTP exporter, run→llm.call→tool.call spans, latency histogram, `estimate_cost` (5 tests), `uar_llm_cost_usd` |
| **H3** RuntimeStep events (per-iteration) | HP3 `emit-runtime-step-events` | ❌ **NOT MET** | **Not implemented — skipped during execution.** Carry-over. |
| **H4** Invoke sycophancy-core post-generation | HP4 `wire-sycophancy-detection` | ✅ **MET** | PR #25; `quality::detect`, `SycophancyFlagged` event, score/flag metrics (4 tests) |
| **H5** Resumable streaming (client) | HP5 `resumable-streaming-client` | ✅ **MET** | PR #26; SSE id capture, `streamSseBlocks` reconnect via `Last-Event-ID`, no duplicate run |
| **H6** Guardrails + mount Cedar layer | HP6 `mount-governance-guardrails` | ✅ **MET** | PR #27; Cedar layer mounted, injection/PII screening, `GuardrailFlagged` event (5 tests) |
| **H7** Eval harness | HP7 (deferred) | ⏸️ **NOT MET (by design)** | Deferred at planning to a dedicated `uar-safety-and-evals` phase (decision R3) |
| **H8** Wire dead metric recorders | folded into HP2 | 🟡 **PARTIAL** | Wired cache-read tokens + latency + cost + confirmed active-sessions live; **sandbox (4) + MCP-status recorders deferred** (no clean data source / health loop) |

**Prerequisite C0 (merge prior-phase branches C1–C4 to `main`): ✅ DONE** — completed before HP1, with `cargo check --features postgres-backend` verified green.

Score: **5 MET / 1 PARTIAL / 2 NOT MET** (1 deferred-by-design, 1 unintended gap).

---

## Delivered changes (merged to `main`)

| Change | PR | Commit area | Tests added |
|---|---|---|---|
| HP1 run-cancellation | #23 | `manager.rs`, `orchestrator` seam, `server.rs` SSE+shutdown, `routes.rs`, frontend stop | 0 dedicated (logic verified via compile + 218 regression) |
| HP2 otlp-tracing-cost | #24 | `telemetry/mod.rs`, `metrics.rs`, `liter_driver.rs`, `catalog.rs`, `mcp/registry.rs`, `manager.rs`, `main.rs` | 5 (`cost_tests`) |
| HP4 wire-sycophancy | #25 | `uar/quality.rs` (new), `server.rs` seam, `events.rs`, `sse.rs`, `metrics.rs` | 4 (`quality::tests`) |
| HP5 resumable-streaming | #26 | `chat-stream-store.ts`, `chat-stream-api.ts` (frontend only) | 0 (logic; typecheck+lint clean) |
| HP6 governance-guardrails | #27 | `uar/guardrails.rs` (new), `server.rs` mount+seam, `policy.rs`, `config.rs`, `events.rs`, `sse.rs`, `metrics.rs` | 5 (`guardrails::tests`) |

---

## Artifact Quality Summary

No `artifact-refiner` logs exist for these changes (the `.refiner/artifacts/` entries belong to other initiatives). QA was performed **inline per change** and is consistent and verifiable:

| Metric | Value |
|---|---|
| Changes shipped / planned (excl. deferred HP7) | 5 / 6 |
| Changes building clean on merged `main` | 5 / 5 |
| New unit tests added across phase | 14 (5 cost + 4 sycophancy + 5 guardrails) |
| Merged-`main` lib test result | **232 passed / 0 failed** |
| New compiler/clippy warnings introduced | 0 (each fixed: `#[derive(Debug)]`, `#[expect]` on casts, backticks) |

### Recurring QA pattern (process)
- **Unrelated `cargo fmt` drift** on `src/uar/api/routes.rs` + `src/uar/rag/ingestion_worker.rs` recurred across HP2/HP4/HP6 — reverted each time to keep diffs surgical (Rule 3). A spawn-task chip was filed to fix the underlying unformatted code on `main` once.
- **Live-env verification deferred** on every change (SIGTERM drain, OTLP collector span tree, mid-stream drop, injection blocking, restrictive-policy 403) — none runnable headlessly; each documented in its `tasks.md`.

---

## Technical debt & deferrals (carried forward)

1. **H3 RuntimeStep events — UNBUILT (highest-priority carry-over).** HP3 was planned and cut-listed but never executed. `NormalizedEvent::RuntimeStep` + per-iteration emission do not exist; the Runtime Console still lacks per-step progress. Re-list as the first item of the next phase or a quick standalone change.
2. **H8 partial** — sandbox metrics (`record_sandbox_*`, 4 recorders) and `set_mcp_server_status` remain dead (need per-runner hooks / an MCP health loop).
3. **Design refinements vs. spec** (sound, documented): HP1 used the consumption-loop `select!` seam instead of a `with_cancellation()` orchestrator builder; HP5 used an async generator instead of `handleBlock`/`readSseStream` extraction. Both satisfy their specs.
4. **Guardrail/quality limits**: sycophancy + injection/PII are detect-only by default; auto-correction and injection-blocking are opt-in/deferred; PII detection is heuristic (false negatives accepted).
5. **Cedar at the tool loop**: `tool_requires_approval` is still the 6-keyword heuristic; `is_tool_allowed` is mounted at HTTP only, not at the orchestrator tool dispatch.
6. **Config liveness**: sycophancy + guardrails read boot-config snapshots; settings-API edits don't affect in-flight runs.
7. **Cost source**: uses the in-repo `ModelCost` catalog; liter-llm's richer cache-aware pricing is unused.

---

## Parity re-score (vs Mastra/Volt/LangGraph/Vercel/Rig)

| Capability | Phase entry | Phase exit |
|---|---|---|
| OTLP tracing w/ span tree | 🔴 | 🟢 |
| Per-LLM latency + cost | 🔴 | 🟢 |
| Run cancellation / abort on disconnect | 🔴 | 🟢 |
| Resumable streaming (client) | 🟡 | 🟢 |
| Response-quality guardrail (sycophancy) | 🔴 | 🟢 |
| Input safety (injection/PII) | 🔴 | 🟡 (heuristic, detect-only) |
| HTTP authorization (Cedar) | 🔴 (unmounted) | 🟢 (mounted, permit-all default) |
| Lifecycle **step** events | 🟡 | 🟡 (**HP3 not done** — RunStart/Tool* only) |
| Evals (model-graded/regression) | 🔴 | 🔴 (deferred) |
| Durable workflows / checkpoint | 🔴 | 🔴 (deferred) |

Net: **6 red→green, 1 yellow→green, 1 red→yellow**; two reds unchanged (deferred), one yellow unchanged (the H3 gap).

---

## Lessons captured

- **Execution drift from the plan is real and must be tracked.** A planned change (HP3) silently fell out of the per-change selection flow. Future phases should reconcile "planned change-sets" vs "shipped" at each merge, not only at reflect.
- **Assessment line numbers go stale fast.** Every change required a fresh Explore pass because prior merges (and the C0 merge) shifted `server.rs`/`manager.rs` substantially. Grounding each change in current code paid off (correct seams, no guesswork).
- **Ground deps in the installed source, not training priors.** OTLP (HTTP vs tonic default feature), liter-llm's `prompt_tokens_details`, and the existing `call_llm` Cedar action were all verified from `~/.cargo` — avoiding wrong APIs and a redundant dependency.
- **"Mount the seam, default-safe" is a strong pattern.** Mounting the Cedar layer with a permit-all default, and detect-only guardrails/sycophancy, delivered the capability without behavior risk (Rule 32) — corrections/enforcement are opt-in.
- **Discovered-architecture refinements beat plan literalism.** HP1's single consumption-loop seam and HP5's generator were simpler/safer than the design's literal approach once the real code shape was known.
- **Headless verification ceiling.** Compile + unit tests + lint catch a lot, but every change left a live-env verification (SIGTERM, OTLP collector, mid-stream drop, policy 403). A dev-env smoke pass should gate "fully done."

---

## Recommended focus for next phase

1. **Close H3 first** — `emit-runtime-step-events` (small, unblocked; the one unfinished planned change). Could be a fast standalone change before opening a new phase.
2. **`uar-safety-and-evals` phase** (the genuine frontier): the eval harness (HP7), sycophancy auto-correction, injection-blocking + PII-block mode, and the `tool_requires_approval` → Cedar `is_tool_allowed` migration at the tool loop.
3. **Finish H8** — wire the sandbox + MCP-health recorders (needs per-runner hooks + a health loop).
4. **Durable workflows / checkpointing** — its own phase (still deferred).
5. **Live-env smoke harness** — make the deferred manual verifications runnable (SIGTERM drain, OTLP collector, mid-stream drop) so "done" includes runtime proof.

---

## Reflection status

- Goals scored honestly: 5 MET / 1 PARTIAL / 2 NOT MET (H3 = unintended gap, surfaced; H7 = planned deferral).
- All 5 shipped changes merged (PRs #23–#27); merged `main` builds + 232 tests pass.
- Carry-over seeds written for the next phase (H3 first, then `uar-safety-and-evals`).
- Ready for `/kbd-new-phase`.
