# Reflection: uar-production-readiness-gaps

**Phase:** `uar-production-readiness-gaps`
**Closed:** 2026-06-02
**Backend:** OpenSpec
**Evolver:** none (active evolution `uar-production-readiness-2026-04` was assess-only; no bridge to write)

---

## Goal Achievement

| Goal | Description | Status | Evidence |
|---|---|---|---|
| G1 | Update `prometheus_parking_lot` reference to latest upstream | ✅ **MET** | `Cargo.toml` rev bumped to `ebb7c3c` (C1); `cargo fetch` clean |
| G2 | Adopt new APIs for graceful shutdown + stuck-task handling | ✅ **MET** | `ShutdownHandle`/`wait_for_signal()` wired; `CancellationToken` in executor; `deadline_ms` populated (C1) |
| G3 | Fix broken shutdown UX — no more `kill` | ✅ **MET** | SIGTERM → `wait_for_signal()` → `pool.shutdown()` → Axum oneshot; pool is single shared instance callable via `Arc` (C1) |
| G4 | Config/env/YAML → datastore → UI — truthful provider/model display | ✅ **MET** | `seed_providers_from_registry` upserts every boot; `provider_keys` shortcuts wired; `configured` requires key presence; configured-models shows operator selection not full catalog (C2) |
| G5 | Built-in agents discoverable + chat-able; agent switching works | ✅ **MET** | `seed_builtin_agents()` persists both at every boot; `agent_id` in chat request body at priority 1; selector renders unconditionally (C3) |
| G6 | Close UI-vs-reality gap (Runtime Console was a dead facade) | ✅ **MET** | `to_runtime_entity_event()` emits `runtime.*` SSE events from live runs; Approve/Deny buttons wired to real `POST /api/uar/runs/{run_id}/approval` endpoint; DEV replay was already env-gated (C4) |
| G7 | Harness parity with Mastra/Volt/peers (partial — this phase) | 🟡 **PARTIAL** | Blocking defect (G1-G3), config truth (G4), agent reliability (G5), console liveness (G6) all closed. OTel tracing, lifecycle hooks from parking-lot bus, durable workflows, evals/guardrails remain — explicitly deferred to `uar-harness-parity` |

**Overall: 6/7 goals MET, 1/7 PARTIAL. Minimum releasable criteria (C1→C4 critical path) satisfied.**

---

## Delivered Changes

| # | Change ID | Branch | Commit | Tests | Status |
|---|---|---|---|---|---|
| C1 | `fix-worker-pool-graceful-shutdown` | `fix/worker-pool-graceful-shutdown` | `86bb7ab` | 3/3 ingestion + 218 lib | ✅ done |
| C2 | `make-config-authoritative-on-boot` | `fix/make-config-authoritative-on-boot` | `c41da57` | 218/218 lib | ✅ done |
| C3 | `persist-builtin-agents` | `fix/persist-builtin-agents` | `f2d19dc` | 218/218 lib | ✅ done |
| C4 | `wire-runtime-console-events` | `fix/wire-runtime-console-events` | `50d4a23` | 218/218 lib | ✅ done |

All 4 branches are committed in their respective worktrees under `~/.claude/worktrees/`. Ready for PR creation and merge to `main`.

---

## Artifact Quality Summary

No artifact-refiner runs were performed for this phase's changes (none of the 4 change IDs appear in `.refiner/artifacts/`). The refiner logs that exist are from prior phases.

| Metric | Value |
|---|---|
| Changes with QA gate (artifact-refiner) | 0/4 |
| Changes with `cargo check` clean | 4/4 |
| Changes with `cargo test --lib` green | 4/4 (218/218 each) |
| Pre-existing test failures carried forward | `config_integration` (4 tests, confirmed pre-existing on `main` before any C1 changes) |

**QA note:** The changes skip artifact-refiner because each touched ≥3 files but the refiner tool was not invoked between changes. Recommended for next phase: wire artifact-refiner into the change-done gate.

---

## Scope Deltas (planned vs. delivered)

| Item | Planned | Delivered | Note |
|---|---|---|---|
| `RuntimeStep` emission (C4) | Emit per-step events | Not delivered | No `RuntimeStep` NormalizedEvent variant; requires orchestrator instrumentation — deferred |
| Parking-lot Hook bus wiring (C4) | Wire `Hook`/`AuditSink` for task events | Not delivered | Deferred to parity backlog — hooks module not yet in the run path |
| Protocols page gating (C4) | Hide un-backed panels | Partial — left as empty state | DEV replay was already gated; Protocols non-destructive but still shows empty in prod |
| Agent-config POST error surfacing (C3) | Surface swallowed errors | Deferred | Low-risk UX polish; swallowing is defensive, not silent data loss |
| Drift metadata persistence (C2) | Persist `SettingSource`/`is_drift` | Deferred | Requires persistence schema addition; not regressed |
| CLI vs LLM_* precedence fix (C2) | Fix `set_override` ordering | Documented as known limitation, not fixed | Both at `set_override` tier; `UAR_LLM__*` wins as intended for the common case |
| C5 observability (OTel/Prometheus) | Partial scaffold | 0% — disk constraint halted work | Explicitly deferred to next phase |
| C6 parity backlog | Deferred from the start | 0% | Correct — `uar-harness-parity` phase |

---

## Technical Debt Introduced

1. **C1 — manual `tokio::time::sleep` still in Axum shutdown path.** The `with_graceful_shutdown` closure uses `tokio::time::sleep(shutdown_timeout)` after the pool drains. This is correct for HTTP connection draining but the duration is now decoupled from the pool drain (pool drains per-worker in 2s each, then detaches; HTTP connections drain for `shutdown_timeout_secs` after). If `shutdown_timeout_secs` is shorter than pool drain time, HTTP connections close before all workers finish. Suggested fix: plumb the actual pool drain time into the HTTP drain timeout (`ShutdownPolicy::DrainThenCancel { deadline }` carries this).

2. **C2 — write-back-to-YAML deferred (P3).** Admin UI provider/model edits do not survive restart. The known tradeoff is documented, but operators editing through the UI will lose edits. The next iteration should add a "Save to config" action or auto-export on edit.

3. **C2 — CLI vs `LLM_MODEL` precedence ambiguity.** Both `--llm-model` and `LLM_MODEL` use `config::set_override`; last-writer wins (legacy env, line 1033). Low severity (`UAR_LLM__MODEL` works correctly), but contradicts documented precedence. Tracked in tasks.md as P3.

4. **C3 — agent-config POST errors swallowed.** `agent-selector.tsx:121` catches and silently drops the session-config POST error. The request-body `agent_id` is now the authoritative path, so this is defensive-not-lossy, but it obscures observability. Wire error toast in a P3 UX polish pass.

5. **C4 — Protocols page not explicitly gated.** `RuntimeAgUiEvent` / `RuntimeModelRouteDecision` / `RuntimeA2uiSurface` still render as empty states in production. They're non-destructive but waste nav space. Explicit hide flag or feature gate deferred.

6. **Cross-cutting — no artifact-refiner QA gate run.** Four changes shipped without refiner. Should be automated per the kbd-execute QA gate protocol for the next phase.

---

## Lessons Captured

1. **The `Arc<T>` + `T::shutdown(self)` anti-pattern kills graceful shutdown.** `IngestionWorkerPool::shutdown(self)` was dead code for exactly this reason. The pattern to enforce: all pools and services that need shutdown must have `fn shutdown(&self)` (not `self`), and the `Arc` must be retained in a scope that participates in the shutdown sequence.

2. **Dependency pin + dead-code shutdown compound each other.** UAR had both a stale dep (missing the fix) AND the wrong method signature (making the fix unreachable). Either alone would be a bug; together they were invisible for months. The lesson: when bumping a dep that adds lifecycle APIs, *immediately* wire them, not later.

3. **"DB-wins-after-first-boot" requires an explicit product decision, not a default.** The prior behavior emerged from pragmatic "don't overwrite user edits" logic, but it silently made env/YAML edits invisible and was never documented. The R3 decision (env/YAML authoritative on boot) was a one-question clarification that unblocked hours of config-path work. Product decisions that affect persistence behavior should be explicit choices, not code defaults.

4. **Disk pressure from multiple parallel worktrees.** Running 4 active worktrees with Rust builds consumed ~40+ GB of build artifacts and blocked all I/O. Strategy: clean completed worktree targets immediately after committing (before starting the next), or point all worktrees to a shared target dir via `.cargo/config.toml` `target-dir`.

5. **runtime-ingest.ts was already present but starved.** The entire entity schema (`RuntimeRun`, `RuntimeRunStep`, `RuntimeToolCall`, `RuntimeApproval`), the `ingestRuntimeEvent` function, and the event-type map were all production-ready. The only missing piece was the backend emitting the events. The Console façade was not "unimplemented" in the sense of missing frontend code — it was starved of a live event source.

---

## Market Position Update (post-phase)

| Capability | Before | After |
|---|---|---|
| Clean shutdown / no `kill` required | ❌ | ✅ |
| Cancellable stuck ingestion tasks | ❌ | ✅ |
| Config changes reflected in UI without manual re-seed | ❌ | ✅ |
| `OPENAI_API_KEY` shortcut works end-to-end | ❌ | ✅ |
| Built-in agents visible and selectable in UI | ❌ (unreliable) | ✅ |
| Agent selection authoritative on request body | ❌ | ✅ |
| Runtime Console shows live runs | ❌ | ✅ |
| Approve/Deny HITL buttons functional | ❌ (dead buttons) | ✅ |
| OTel/OTLP tracing | ❌ | ❌ (deferred) |
| Lifecycle hooks wired (parking-lot bus) | ❌ | ❌ (deferred) |
| Durable workflows / checkpoint | ❌ | ❌ (deferred) |
| Evals / guardrails | ❌ | ❌ (deferred) |

Revised parity score vs. Mastra/Volt/LangGraph/Vercel/Rig: **~9 green / 5 yellow / 8 red** (was ~5/7/11 at assessment). The hard blockers are closed. The remaining red items are all in the `uar-harness-parity` deferred phase.

---

## Recommended Focus for Next Phase

**Phase `uar-harness-parity`** — the parity backlog deferred from C6:

| Priority | Item | Rationale |
|---|---|---|
| P0 | OTel/OTLP tracing (C5 partial) | Single highest-value missing primitive — needed for production debuggability, cost dashboards, latency analysis. Rig proves feasible in Rust. Wire the existing `kreuzberg` + `tracing` crates into OTLP spans. |
| P1 | Parking-lot `Hook`/`LifecycleEvent` bus wiring | Unlocks per-task telemetry, cancellation-through-tools, and `RuntimeStep` entity emission — all stalled on this one wiring point. |
| P1 | Cancellation propagated through tool calls | Extend the `CancellationToken` from C1 into the LLM orchestrator's tool execution loop (not just ingestion). Closes "stuck tool call" scenario. |
| P2 | Resumable SSE streaming (client reconnect) | VoltAgent has this; users lose context on browser refresh mid-stream. |
| P3 | Config write-back to YAML | Persist admin UI provider/model edits back to config so they survive restart (R3 deferred tradeoff). |
| P3 | Evals scaffold | Model-graded + rule-based; Mastra/Volt have this. Required for "production ready" claim. |
| Deferred | Durable workflows / checkpoint | LangGraph's headline feature; large scope. Phase of its own. |
| Deferred | Guardrails (prompt injection, input/output) | `sycophancy-core` already present; extend to injection defense. |

**Also carry forward:**
- Credentials admin UI (prior carryover from `uar-kreuzberg-v4-migration`)
- Agent-config POST error surfacing (C3 P3 polish)
- Protocols page explicit gating (C4 P3 cleanup)
- Artifact-refiner QA gate automation for future phases

**Recommended immediate action:** Create 4 PRs (one per branch) from the worktrees to `main`. C1 and C2 are independent; C3 and C4 are independent of each other; C4 depends on C1 being merged first (for the hook bus, though C4 doesn't strictly use it yet, the test fixtures share the parking-lot rev).

---

## PR Strategy

```
PR order:
1. C2 make-config-authoritative-on-boot  (standalone, no deps)
2. C1 fix-worker-pool-graceful-shutdown  (standalone, unblocks C4)
3. C3 persist-builtin-agents             (standalone, no deps)
4. C4 wire-runtime-console-events        (depends on C1 parking-lot bump being on main)
```

Branches to PR:
- `fix/make-config-authoritative-on-boot` → `main`
- `fix/worker-pool-graceful-shutdown` → `main`
- `fix/persist-builtin-agents` → `main`
- `fix/wire-runtime-console-events` → `main`
