PLAN: uar-harness-parity
Project: universal-agent-runtime
Date: 2026-06-02
OpenSpec available: YES
Planning model: Opus 4.8 (frontier-class) — satisfies kbd-plan model policy
Changes to implement: 6 (+ 1 non-code merge gate)

---

## Product decisions resolved (this plan is grounded in them)

- **R2 — cancel semantics:** cancel on **last-subscriber-drop** + an explicit `POST /runs/{id}/cancel` endpoint and UI stop button. Naive first-drop cancel is rejected (would kill runs other clients are watching via history replay).
- **R3 — eval harness scope:** **deferred** to a dedicated `uar-safety-and-evals` phase. Not in this phase.
- **R4 — guardrails:** **in-house heuristics + mount the existing Cedar `governance_layer`**; no external moderation service. HP6 stays in this phase.

Scope cut (S-07 / S-03 discipline): the full eval harness (HP7) and any external guard-model integration are explicitly OUT. This phase ships cancellation + observability + step events + sycophancy + resumable client + governance guardrails.

---

## ⚠️ Round 0 — Merge gate (NOT an OpenSpec change)

**C0 — merge prior-phase branches to `main` before any code change below.** The assessment ran against `main` (`8b3c503`), which lacks the prior phase's work. Merge order (per prior reflection `prStrategy`):

```
C2 fix/make-config-authoritative-on-boot
C1 fix/worker-pool-graceful-shutdown        (parking-lot bump; C4 depends on it)
C3 fix/persist-builtin-agents
C4 fix/wire-runtime-console-events           (adds runtime.* events HP3 extends)
```

No `/opsx:new` for this — it is a `git merge` / PR-merge gate. HP1–HP6 assume `main` already contains the ingestion `CancellationToken`, graceful shutdown, and `runtime.*` SSE events. **Do not start Round 1 until C0 is on `main`.**

---

## CHANGE LIST (ordered)

1. **add-run-cancellation**: Per-run `CancellationToken` so client last-disconnect, an explicit cancel endpoint, and shutdown all abort in-flight LLM + tool calls — no orphaned runs.
   - Scope: api | orchestrator | server (SSE) | ui (stop button)
   - Depends on: C0 (merge gate)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Create a `CancellationToken` per run in `start_run` (`manager.rs:441`), store in `RunStreamState`, add `RunManager::cancel_run` + `POST /api/uar/runs/{id}/cancel`. Thread a child token into `Orchestrator::chat_with_history` via a `with_cancellation()` builder mirroring the approval-gate seam (`orchestrator.rs:237`). `tokio::select!` the token against the driver stream call (`orchestrator.rs:376`), the consumption loop (`orchestrator.rs:449`), and each MCP/native/sandbox tool `.await` (`orchestrator.rs:685/725/730`); `is_cancelled()` check atop the tool loop (`orchestrator.rs:600`). SSE drop-guard in `server.rs:3895` cancels only on **last-receiver-drop** (count subscribers; respect `history_since` late-joiners). Derive run tokens from a root token cancelled by `shutdown_signal` (`server.rs:1140`). Add UI stop button + cancelled-run terminal event.

2. **wire-otlp-tracing-and-cost**: Export a real OTLP span tree (run → LLM → tool) with per-LLM latency and per-request cost; light up the 6 dead metric recorders.
   - Scope: telemetry | orchestrator | llm driver | mcp registry
   - Depends on: add-run-cancellation (shared orchestrator surface — sequence after to avoid conflict)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Wire an `opentelemetry_otlp` exporter + `TracerProvider` + `tracing_opentelemetry::layer()` in `telemetry::init` (currently fmt-only), gated on the dead `llm.tracing` flag (`config.rs:1202`) + an OTLP endpoint env var. Add `#[instrument]`/`span!` to the LLM driver call (`liter_driver.rs`) and tool dispatch (`mcp/registry.rs:387`) so the single `start_run` span (`manager.rs:420`) gains children. Wrap the LLM call with `Instant`/`.elapsed()` for per-call latency. Compute cost from the existing pricing struct (`uar/llm/schema.rs:14`) when `cost_tracking` is on (`config.rs:1201`) and emit `uar_llm_cost_usd`. Cheap wins (H8): wire `record_cache_tokens` (data already at `anthropic_streaming.rs:228`), the 4 sandbox recorders, and `set_mcp_server_status`. Put `run_id` on span attributes, NOT metric labels (R5 cardinality).

3. **emit-runtime-step-events**: Per-iteration `RuntimeStep` events through the existing `RunEventEmitter` so the Runtime Console shows orchestrator progress — NOT a new bus.
   - Scope: orchestrator | event model | sse | ui (console)
   - Depends on: add-run-cancellation (shared orchestrator loop); C4 `runtime.*` events (via C0)
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Details: Add `NormalizedEvent::RuntimeStep { run_id, step, kind }` and emit it per orchestrator iteration (`orchestrator.rs:348`) through the existing broadcast+replay `RunEventEmitter` (reuses the 512-event history). Map to a `runtime.*` SSE event (extends C4) and render in the Runtime Console. **Explicitly NOT** wiring the parking-lot `HookBus` — it would duplicate this emitter.

4. **wire-sycophancy-detection**: Actually invoke `sycophancy-core` post-generation and route the score against the already-configured thresholds.
   - Scope: orchestrator (response path) | metrics
   - Depends on: NONE (independent files; coordinate with HP1 on response-path edits)
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Details: `sycophancy-core` is a declared-but-dead dependency (`Cargo.toml:111`). Invoke `sycophancy_core::skill::detector::Detector` on the completed assistant response in the chat/orchestrator path, route the score against the existing `auto_correct_threshold`/`reflect_threshold` (`config.rs:1657-1700`, already editable via `api/settings.rs:241`), and emit a quality event/metric. No new config surface — consume what exists.

5. **resumable-streaming-client**: Wire the frontend to the already-complete server resume endpoint so a mid-stream drop reconnects instead of re-POSTing a duplicate run.
   - Scope: frontend (chat-stream-store) | server (one response header)
   - Depends on: NONE (fully independent)
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Details: Server side is done (512-event buffer, monotonic ids, `GET /api/uar/runs/{id}/stream?last_event_id=` honouring `Last-Event-ID`, `routes.rs:63-94`). Expose `run_id` as a response header next to `x-uar-session-id` (`server.rs:4278`). Frontend: capture `run_id`, stop discarding the SSE `id:` line (`chat-stream-store.ts:212`), track last-event-id, and on mid-stream drop call the resume GET instead of re-POST (`chat-stream-store.ts:596`); fix retry being disabled after first chunk (`sawFirstStreamChunk`). Detect gap when a resume id predates the 512-event window. (UI/UX routing per CLAUDE.md applies to the frontend edits.)

6. **mount-governance-guardrails**: Mount the defined-but-unmounted Cedar `governance_layer`, add `call_llm`/`validate_output` actions, and add in-house input injection/PII screening on the chat path.
   - Scope: server (middleware) | governance (Cedar) | chat path
   - Depends on: NONE (independent; touches server middleware stack)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Mount the existing `governance_layer` middleware (`governance/middleware.rs:48`, currently never mounted) into the chat path. Add Cedar `call_llm` + `validate_output` actions to `GovernanceEngine`; keep the permit-all default but make the seam live. Add lightweight in-house heuristic injection/jailbreak/PII screening on input (and basic output validation) in `api_chat_completion` (`server.rs:3630`). No external moderation service (R4). Note: this does NOT replace the brittle 6-keyword tool-approval heuristic (`manager.rs:249`) — that Cedar migration is tracked separately.

---

## EXECUTION ROUND ORDER

- **Round 0 (gate):** C0 merge prior-phase branches to `main`.
- **Round 1 (parallel, independent files):** `add-run-cancellation`, `resumable-streaming-client`, `mount-governance-guardrails`, `wire-sycophancy-detection`.
  - Coordination: HP1 and HP4 both touch the orchestrator response path — run HP1 first or in an isolated worktree; rebase HP4 on it.
- **Round 2 (depends on HP1's orchestrator surface):** `emit-runtime-step-events`, then `wire-otlp-tracing-and-cost`.
  - HP2's `#[instrument]` additions are mostly non-conflicting; sequence after HP3 to keep the orchestrator loop diff clean.

---

## DEFERRED (out of this phase)

- **HP7 eval harness** → dedicated `uar-safety-and-evals` phase (R3).
- **Durable workflows / checkpointing** → own phase.
- **Config write-back to YAML** → own change (C2 carryover).
- **Tool-approval Cedar migration** (replace the 6-keyword heuristic) → fold into `uar-safety-and-evals`.
- **Parking-lot `HookBus`** → will NOT be built (redundant with `RunEventEmitter`).

---

## COMMANDS TO RUN

```
# Round 0 — merge gate first (no opsx change):
#   merge C2 → C1 → C3 → C4 to main, verify cargo build + cargo test green.

/opsx:new add-run-cancellation
/opsx:new wire-otlp-tracing-and-cost
/opsx:new emit-runtime-step-events
/opsx:new wire-sycophancy-detection
/opsx:new resumable-streaming-client
/opsx:new mount-governance-guardrails
```

---

## Sycophancy self-check

- **S-02 (agreement without grounding):** every change cites file:line evidence from the assessment; no feasibility assumed beyond it. The merge gate (C0) surfaces that the assessment's baseline is unmerged rather than papering over it.
- **S-07 (scope creep):** the eval harness, external guard model, durable workflows, config write-back, and tool-approval Cedar migration are all explicitly cut to keep the phase focused on the P0 cancellation + observability work.
- **S-03 (caveat collapse):** trade-offs surfaced — cancellation multi-subscriber risk (R2), OTel cardinality (R5), `cost_tracking` default-false (O1), orchestrator-file contention between HP1/HP3/HP4, and the dependency on C0 being merged first.

PLAN COMPLETE
