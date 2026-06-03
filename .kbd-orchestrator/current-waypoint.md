# Current Waypoint

- Phase: `uar-harness-parity` **(reflect_complete)**
- Previous phase: `uar-production-readiness-gaps`
- Backend: OpenSpec
- Status: `reflect_complete`
- Progress: **5 / 6 changes shipped** (PRs #23–#27 merged) · 1 planned change not built (H3)
- Exact next command: `/kbd-new-phase`
- Reflection: [reflection.md](phases/uar-harness-parity/reflection.md)
- Merged `main` HEAD: `ea01958` · 232 lib tests pass
- Updated at: 2026-06-03

## Phase arc outcome

**5 MET / 1 PARTIAL / 2 NOT MET.**

Shipped + merged: run cancellation (H1, #23), OTLP tracing + per-LLM latency + cost (H2, #24), sycophancy detection (H4, #25), resumable streaming client (H5, #26), mounted Cedar layer + injection/PII guardrails (H6, #27). H8 PARTIAL (cache/latency/cost/sessions wired; sandbox + MCP recorders deferred).

**NOT MET:** **H3 `emit-runtime-step-events` was never built** — skipped during execution (carry-over, top priority). H7 eval harness deferred by design.

Parity vs Mastra/Volt/LangGraph/Vercel/Rig: **6 red→green, 1 yellow→green, 1 red→yellow** (injection/PII heuristic). Lifecycle **step** events stayed yellow (the H3 gap).

## Goal scoreboard

| Goal | Change | Status |
|---|---|---|
| H1 cancellation | HP1 #23 | ✅ MET |
| H2 OTLP + latency + cost | HP2 #24 | ✅ MET |
| H3 runtime-step events | HP3 | ❌ NOT BUILT (carry-over) |
| H4 sycophancy detection | HP4 #25 | ✅ MET |
| H5 resumable streaming | HP5 #26 | ✅ MET |
| H6 guardrails + Cedar mount | HP6 #27 | ✅ MET |
| H7 eval harness | — | ⏸️ deferred (by design) |
| H8 dead metric recorders | in HP2 | 🟡 PARTIAL |

## Carry-over / debt

1. **H3 `emit-runtime-step-events`** — unbuilt planned change; do first.
2. Unformatted `routes.rs` + `ingestion_worker.rs` on `main` — spawn-task chip filed (fmt-only).
3. Sandbox (4) + `set_mcp_server_status` recorders still dead.
4. `tool_requires_approval` still heuristic; Cedar `is_tool_allowed` mounted at HTTP only.
5. Live-env verifications deferred across all changes (SIGTERM, OTLP collector, mid-stream drop, injection 403).

## Recommended next phase

- **First:** close **H3** (small, unblocked) — standalone change or open the next phase with it.
- **Then:** `uar-safety-and-evals` — eval harness, sycophancy auto-correction, injection-blocking + PII-block, tool-loop Cedar gating.
- Finish **H8**; build a **live-env smoke harness**; config liveness; durable workflows (own phase).
