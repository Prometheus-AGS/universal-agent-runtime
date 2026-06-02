# Assessment: UAR Harness Parity

**Phase:** `uar-harness-parity`
**Date:** 2026-06-02
**Project:** Universal Agent Runtime (`src/` + `frontend/`)
**Assessed by:** kbd-assess + 4 parallel codebase subagents
**Assessment discipline:** Sycophancy-corrected — findings stated bluntly, file:line-cited, and re-prioritized against ground truth even where that contradicts the prior phase's reflection seeds.

---

## ⚠️ Hard Prerequisite (read first)

This assessment was run against `main` (base `8b3c503`). **The four branches from the prior phase (`uar-production-readiness-gaps`) are NOT yet merged:**

- `fix/make-config-authoritative-on-boot` (`c41da57`)
- `fix/worker-pool-graceful-shutdown` (`86bb7ab`) — adds the ingestion `CancellationToken` + graceful shutdown
- `fix/persist-builtin-agents` (`f2d19dc`)
- `fix/wire-runtime-console-events` (`50d4a23`) — adds `to_runtime_entity_event` + `runtime.*` SSE events

Several findings below ("no `CancellationToken` in `src/`", "no `runtime.*` events") are **true for `main` but already partially addressed in those branches.** This phase's work *extends* them. **Merge C1→C2→C3→C4 to `main` before starting execution of this phase**, or the cancellation/observability work will collide. The phase plan must treat the merge as step 0.

---

## Executive Summary

The prior phase closed the production *blockers*. This phase is about *parity* with Mastra/VoltAgent/LangGraph. The four investigations reveal that the gap is **smaller in two areas and larger in one** than the reflection seeds assumed:

1. **Observability is half-built, not absent.** Prometheus `/metrics` is wired and auth-exempt (`server.rs:908`); JSON logging is done (`telemetry/mod.rs`); 4 of 10 metric recorders are live. **The real gap is OTLP tracing: the `opentelemetry*` deps are declared in `Cargo.toml` but used nowhere in `src/`** — and there is exactly **one** `#[instrument]` on the whole run path (`manager.rs:420` on `start_run`), so even adding an exporter would emit a flat single-span trace. Cost is computable (pricing struct + `cost_tracking` flag exist) but never computed.

2. **Run cancellation is a genuine production bug, not a nicety — promote to P0.** There is **zero** cancellation on the run path. The agent task is `tokio::spawn`ed with its `JoinHandle` discarded (`manager.rs:887`). When an SSE client disconnects, the run is **orphaned, not cancelled** — it keeps calling the LLM and executing tools (including destructive MCP/native tools matched by the `delete|remove|write|drop|truncate|destroy` heuristic) and burning tokens after the user has left. This is a cost + safety defect, not a parity gap.

3. **Resumable streaming is ~90% done server-side — downgrade to a small frontend task.** `RunManager` already has a 512-event ring buffer with monotonic `u64` ids, `history_since(last_id)`, AND a working `GET /api/uar/runs/{id}/stream?last_event_id=` endpoint that honours the `Last-Event-ID` header and merges replay+live with dedup (`routes.rs:63-94`). The gap is **entirely client-side**: the frontend never captures the server `run_id`, discards the SSE `id:` field, and on mid-stream drop re-POSTs the message (duplicate run) instead of calling the resume endpoint.

4. **Evals + guardrails are the largest TRUE gap — and `sycophancy-core` is a dead dependency.** Contrary to the reflection ("sycophancy-core already present"), it is declared in `Cargo.toml:111` but **never imported or invoked in `src/`** — only config scaffolding (`SycophancyConfig`) with no consumer exists. There is no eval harness, no prompt-injection defense, no input/output sanitization on the chat path. The Cedar `GovernanceEngine` is well-built but defaults to permit-all, its HTTP `governance_layer` middleware is **defined and never mounted**, and `is_tool_allowed` is never called at runtime.

5. **KILL one reflection seed: do NOT wire the parking-lot `HookBus`.** The audit is unambiguous — it's unused, gated behind a feature UAR doesn't enable (`tokio-runtime` only), and would **duplicate the existing `RunEventEmitter` broadcast+replay bus** that already emits `RunStart`/`ToolStart`/`ToolEnd`/`RunDone` with a 512-event history. The useful part of that seed (per-step events) is achieved by adding a `RuntimeStep` variant to `NormalizedEvent`, not a second bus.

**Net:** the phase shrinks in two dimensions (resumable streaming, observability), grows in one (cancellation is now P0), drops one seed (HookBus), and the genuine frontier work is evals + guardrails.

**Complexity:** Medium-High · **Risk:** Medium · **Recommended decomposition:** see §7.

---

## Goals (derived from reflection seeds, re-prioritized against findings)

| # | Goal | Seed origin | Revised priority |
|---|---|---|---|
| H1 | Run cancellation — client disconnect + shutdown abort in-flight LLM/tool calls; no orphaned runs | "P1 cancellation through tool calls" | **P0 (promoted — correctness/cost/safety)** |
| H2 | OTLP tracing exported with a real span tree (run→LLM→tool) + per-LLM latency + per-request cost | "P0 OTel tracing" | **P0** |
| H3 | `RuntimeStep` events per orchestrator iteration via existing `RunEventEmitter` | "P1 HookBus" (corrected) | P1 |
| H4 | Resumable streaming wired client-side (server already complete) | "P2 resumable streaming" | **P2 (downgraded — frontend-only)** |
| H5 | Actually invoke `sycophancy-core` post-generation (response-quality guardrail runs) | "P3 guardrails" | P1 |
| H6 | Input/output guardrails (injection defense) + mount the Cedar `governance_layer` | "P3 evals/guardrails" | P2 |
| H7 | Eval harness (model-graded/rule-based, persisted, regression over time) | "P3 evals scaffold" | **P2 or own phase (product decision)** |
| H8 | Cheap wins: wire the 6 dead metric recorders | new (from audit) | P3 |
| — | Durable workflows / checkpointing | "deferred" | Deferred → own phase |
| — | Config write-back to YAML | "P3" (C2 carryover) | Deferred → own change |

---

## D1 — Observability (H2, H8)

**Headline: Prometheus + logging built; OTLP tracing entirely unimplemented despite deps present.**

### What EXISTS
- `GET /metrics` registered (`server.rs:908` → `api_metrics` `server.rs:1695`), renders Prometheus text, auth-exempt (`security/middleware.rs:73`). Clean pull-model.
- JSON/compact/pretty logging done (`telemetry/mod.rs:15-57`).
- 4/10 metric recorders WIRED: `record_request` (`server.rs:1027`), `record_llm_tokens` (`liter_driver.rs:214`), `record_tool_call` (`mcp/registry.rs:387`), `set_active_sessions` (`thread.rs:325/356/391`).
- Pricing model struct (`uar/llm/schema.rs:14`) and `cost_tracking` flag (`config.rs:1201`) exist.
- Anthropic cache tokens already parsed (`anthropic_streaming.rs:228-250`).

### What's MISSING (the gap, in order)
1. **OTLP pipeline init** — `telemetry/mod.rs::init` builds only `fmt` layers. No `opentelemetry_otlp` exporter, no `TracerProvider`, no `tracing_opentelemetry::layer()`. The `opentelemetry*` deps (`Cargo.toml:198-201`) are dead weight. The `llm.tracing: bool` flag (`config.rs:1202`) is a dead toggle — never read. (~30 lines to wire.)
2. **Child spans on the run path** — exactly ONE `#[instrument]` exists (`manager.rs:420`). Zero `span!`/`#[instrument]` in `src/llm/`, `src/mcp/`, `src/uar/runtime/` beyond it. Even with an exporter, a run exports as one flat span. **This is the largest correctness gap for a usable waterfall** — add spans to the LLM driver call (`liter_driver.rs`) and tool dispatch (`mcp/registry.rs:387`).
3. **Per-LLM-call latency** — only HTTP latency is recorded. No `Instant`/`.elapsed()` around the LLM call.
4. **Cost computation** — `cost_tracking` never read; no `tokens × pricing`; no `uar_llm_cost_usd` metric. Data + struct exist; only the multiply+emit is missing.
5. **6 dead recorders** (H8): `record_cache_tokens` (data exists at `anthropic_streaming.rs:228`!), all 4 sandbox recorders, `set_mcp_server_status` — defined, never called.

---

## D2 — Run cancellation (H1) — **PROMOTED TO P0**

**Headline: client disconnect orphans the run; it keeps executing tools and burning tokens.**

### Evidence
- No `CancellationToken` / `AbortHandle` / disconnect detection anywhere in `src/` (on `main`).
- Agent loop is `tokio::spawn`ed at `manager.rs:887` with the `JoinHandle` **discarded**; `RunManager` has no abort API.
- SSE handler (`server.rs:~3895-4275`) tail is a bare `while let Ok(event) = rx.recv().await` with **no `tokio::select!` on disconnect, no drop guard**. Dropping the client drops the `broadcast::Receiver`, but the producer task runs to completion. `sender.send` errors are ignored (`manager.rs:66`).
- On shutdown, `shutdown_signal` just sleeps a grace timer (`server.rs:1140`); in-flight runs are not signalled, only killed at process teardown.

### The precedent to model on (NOT parking-lot)
The **approval gate** is the existing out-of-band, run-id-keyed interrupt: `pending_approvals: HashMap<run_id, oneshot::Sender<bool>>` (`manager.rs:105`) + `with_tool_approval_gate` (`orchestrator.rs:237`) + `resolve_approval` (`manager.rs:407`). It already pauses the orchestrator loop on an external signal. **Cancellation is structurally identical** and should reuse this seam.

### Integration points
1. Create a `CancellationToken` per run in `start_run` (`manager.rs:441`); store in `RunStreamState`; add `RunManager::cancel_run(run_id)`.
2. Thread a child token into `Orchestrator::chat_with_history` (`orchestrator.rs:285`) via a `with_cancellation()` builder mirroring the approval gate.
3. `tokio::select!` the token against the driver `stream` call (`orchestrator.rs:376`) + the consumption loop (`orchestrator.rs:449`).
4. `is_cancelled()` check at the top of the tool loop (`orchestrator.rs:600`) and `select!` around each MCP/native/sandbox `.await` (`orchestrator.rs:725/730/685`).
5. Signal on client disconnect via a drop-guard in the SSE stream closure (`server.rs:3895`) — **but gate on last-receiver-drop**, since the broadcast supports multiple subscribers + late joiners via `history_since` (a naive per-receiver guard would cancel runs other clients are watching).
6. Derive run tokens from a root token cancelled by `shutdown_signal` for graceful shutdown.

---

## D3 — Resumable streaming (H4) — **DOWNGRADED to frontend-only**

**Headline: server-side is complete; only the client doesn't use it.**

### What EXISTS (server)
- 512-event ring buffer per run, monotonic `u64` ids (`manager.rs:27-39`).
- `history_since(run_id, last_event_id)` — id-filtered replay (`manager.rs:1300`).
- `GET /api/uar/runs/{id}/stream?last_event_id=` (`routes.rs:63-94`) — reads `Last-Event-ID` header, merges replay+live with dedup. **This is a correct, working resume endpoint.**

### The gap (client only)
- Frontend streams via `POST /api/chat/completion` + `fetch`/`ReadableStream` (`chat-stream-store.ts:596`), never `EventSource`, never the resume GET.
- `parseSseBlock` discards the `id:` line (`chat-stream-store.ts:212`); no last-event-id tracked.
- Server `run_id` never captured client-side (UI uses local `run-${Date.now()}`).
- Retry re-POSTs the message → **duplicate run**; and retry is *disabled after first chunk* (`sawFirstStreamChunk`), exactly when resume matters.

### Minimal fix
Expose `run_id` as a response header (next to `x-uar-session-id` at `server.rs:4278`); parse + track SSE `id:`; on mid-stream drop, `GET /runs/{run_id}/stream` with `Last-Event-ID` instead of re-POST. ~half-day frontend change. Optionally add gap detection when a resume id predates the 512-event window.

---

## D4 — Evals + Guardrails (H5, H6, H7) — **the genuine frontier**

### Sycophancy (H5) — dead dependency
`sycophancy-core` (`Cargo.toml:111`) is **never imported/invoked in `src/`**. Only `SycophancyConfig` (`config.rs:138-140, 1657-1700`) + settings registration (`manager.rs:2200`) + REST surface (`api/settings.rs:241`) exist — thresholds are editable but **nothing reads them**. The detector is wired only in the separate `prometheus-cli` tree, not in `api_chat_completion`. **H5 is bounded:** invoke `sycophancy_core::skill::detector::Detector` post-generation, route the score against `auto_correct_threshold`/`reflect_threshold`.

### Eval harness (H7) — none
No model-graded/rule-based/golden/faithfulness/relevance/toxicity scoring in `src/`. `src/testing/` is CI-flakiness analytics and **isn't even compiled** (`mod testing` not declared in `lib.rs`/`main.rs`). This is the largest scope item — **product decision: full harness this phase vs scaffold-only vs own phase.**

### Guardrails / injection defense (H6) — absent
`api_chat_completion` (`server.rs:3630`) does only structural validation. No prompt-injection/jailbreak/PII/moderation/sanitization on input or output. Middleware stack is auth→body-limit→timeout→rate-limit→CORS→metrics — none inspect content.

### GovernanceEngine — built but disconnected
Cedar engine (`governance/engine.rs`) is well-built (hot-reload, tool/skill/llm actions) but: **defaults to permit-all** (`server.rs:453`), the `governance_layer` middleware (`governance/middleware.rs:48`) is **defined and never mounted**, `is_tool_allowed` is **never called at runtime** (only skill-mutation gating is live, `skills/service.rs:308`). It is the obvious extension point for a `call_llm`/`validate_output` guardrail — add the action and finally mount the layer.

### Tool approval — brittle heuristic
`tool_requires_approval` (`manager.rs:249`) is a 6-keyword substring match (`delete|remove|write|drop|truncate|destroy`) — trivially bypassed (`purge`, `rm_files` not caught). Code comment admits "will be replaced by Cedar." It is the only live tool guardrail.

---

## §5 — Production-Readiness Re-Score (vs Mastra/Volt/LangGraph/Vercel/Rig)

| Capability | At parity-phase entry | After this phase (if H1–H6 land) |
|---|---|---|
| OTel/OTLP tracing w/ span tree | 🔴 (deps unused) | 🟢 |
| Prometheus metrics | 🟢 (already done) | 🟢 |
| Per-LLM latency + cost | 🔴 | 🟢 |
| Run cancellation / abort on disconnect | 🔴 (orphaned runs) | 🟢 |
| Lifecycle/step events | 🟡 (RunStart/Tool* exist; no Step) | 🟢 |
| Resumable streaming | 🟡 (server done, client not) | 🟢 |
| Response-quality guardrail (sycophancy) | 🔴 (dead dep) | 🟢 |
| Input/output safety (injection/PII) | 🔴 | 🟡 (basic) |
| Evals (model-graded/rule/regression) | 🔴 | 🟡 or 🔴 (scope decision) |
| Durable workflows / checkpoint | 🔴 | 🔴 (deferred) |

---

## §7 — Recommended Decomposition (feeds `/kbd-plan`)

Ordered by corrected leverage. **Change 0 is the merge prerequisite.**

0. **C0 — merge prior-phase branches** to `main` (C1→C2→C3→C4). Not a code change; a gate. Everything below assumes `main` has the ingestion `CancellationToken`, graceful shutdown, and `runtime.*` events.
1. **HP1 — run cancellation on disconnect + shutdown** (H1, P0). Per-run `CancellationToken` modeled on the approval-gate seam; `select!` around LLM + tool `.await`s; drop-guard gated on last-receiver-drop; root token cancelled by `shutdown_signal`. *Highest leverage — fixes a live cost/safety bug.*
2. **HP2 — OTLP tracing + run-path spans + latency + cost** (H2, P0). Wire OTLP exporter in `telemetry::init` (gate on `llm.tracing` + endpoint env); add `#[instrument]` to LLM driver + tool dispatch; wrap LLM call with latency; compute cost from existing pricing struct when `cost_tracking` on. Include H8 cheap wins (6 dead recorders).
3. **HP3 — `RuntimeStep` events** (H3, P1). Add `NormalizedEvent::RuntimeStep { run_id, step, kind }`; emit per orchestrator iteration (`orchestrator.rs:348`); flows through existing emitter → SSE/Console. **No new bus.**
4. **HP4 — wire sycophancy-core post-generation** (H5, P1). Invoke detector in the chat/orchestrator response path; route score against the existing thresholds; emit a quality event/metric.
5. **HP5 — resumable streaming (client)** (H4, P2). Expose `run_id` header; track SSE id; reconnect via the existing resume GET instead of re-POST; fix retry gating.
6. **HP6 — guardrails: mount governance_layer + injection/PII checks** (H6, P2). Mount the existing `governance_layer`; add a `call_llm`/`validate_output` Cedar action; add basic input injection/PII screening on the chat path.
7. **HP7 — eval harness** (H7, P2 or own phase — **product decision required**). If in-scope: rule-based + LLM-as-judge scorers, prompt suites, persisted metrics.

**Suggested cut for THIS phase:** C0 (merge) → HP1 → HP2 → HP3 → HP4. HP5 if time. HP6 + HP7 are strong candidates to split into a dedicated `uar-safety-and-evals` phase given their combined scope.

**Explicitly NOT doing:** wiring the parking-lot `HookBus` (redundant with `RunEventEmitter`); durable workflows (own phase); config write-back to YAML (own change).

---

## Risks & Open Questions

- **R1 — merge prerequisite (C0).** If the prior branches don't merge first, HP1/HP2/HP3 collide with C1/C4. Hard-gate the plan on the merge.
- **R2 — cancellation + multi-subscriber.** The broadcast supports multiple receivers + late joiners (`history_since`). Disconnect→cancel must key on *last-receiver-drop* or an explicit endpoint, not first-drop, or it will kill runs other clients are watching. **Product/UX decision:** cancel-on-last-disconnect vs explicit "stop" button only.
- **R3 — eval harness scope.** Full harness is large. **Decision needed:** in-phase scaffold, in-phase full, or dedicated `uar-safety-and-evals` phase. Recommend the latter.
- **R4 — guardrail build-vs-buy.** In-house injection/PII detection vs an external model/service. Affects HP6 scope.
- **R5 — OTel cardinality.** Per-model/provider/path labels are fine; avoid per-run-id labels on metrics (unbounded cardinality). Use span attributes for run_id, not metric labels.
- **O1 — `cost_tracking` default is `false`.** Cost computation should respect it but the default may need flipping to `true` for the cost metric to be useful out-of-the-box.

---

## Assessment status

- Dimensions assessed: 4 (observability, cancellation, resumable streaming, evals/guardrails) — all file:line-grounded.
- Reflection seeds corrected: 3 (HookBus killed, resumable downgraded, cancellation promoted to P0; sycophancy "present" → "dead dep").
- Blocking prerequisite identified: **yes** (merge prior-phase branches first).
- Product decisions flagged: R2 (cancel semantics), R3 (eval scope), R4 (guardrail build-vs-buy).
- Ready for `/kbd-plan`: **yes** — proposed C0 + HP1–HP7 in §7.
