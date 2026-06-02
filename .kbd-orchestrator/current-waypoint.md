# Current Waypoint

- Phase: `uar-harness-parity` **(planned)**
- Previous phase: `uar-production-readiness-gaps` *(6/7 goals met)*
- Backend: OpenSpec
- Status: `planned`
- Progress: **0 / 6 changes** · plan complete
- Active change: none
- Exact next command: **Round 0 merge gate** (C2→C1→C3→C4 to `main`), then `/opsx:new add-run-cancellation`
- Plan: [plan.md](phases/uar-harness-parity/plan.md) · Assessment: [assessment.md](phases/uar-harness-parity/assessment.md)
- Updated at: 2026-06-02T00:00:00Z

## ⚠️ Round 0 — Merge gate (do first, not an OpenSpec change)

Assessment was run against `main` (`8b3c503`), which lacks the prior phase's work. Merge **C2 → C1 → C3 → C4** to `main` and verify `cargo build` + `cargo test` green BEFORE starting Round 1. HP1/HP2/HP3 assume `main` has the ingestion `CancellationToken`, graceful shutdown, and `runtime.*` SSE events.

## Product decisions resolved

| ID | Decision |
|---|---|
| R2 cancel semantics | Cancel on **last-subscriber-drop** + explicit `POST /runs/{id}/cancel` + UI stop button |
| R3 eval scope | **Deferred** to dedicated `uar-safety-and-evals` phase |
| R4 guardrails | **In-house heuristics + mount existing Cedar `governance_layer`** (no external service) |

## Change roster (ordered)

| # | Change | Round | Complexity | Model | Value | Agent |
|---|---|---|---|---|---|---|
| 1 | `add-run-cancellation` | 1 | L / High | frontier | HIGH | Claude Code |
| 2 | `wire-otlp-tracing-and-cost` | 2 | L / High | frontier | HIGH | Claude Code |
| 3 | `emit-runtime-step-events` | 2 | M / Med | medium | MED | Codex |
| 4 | `wire-sycophancy-detection` | 1 | M / Med | medium | MED | Codex |
| 5 | `resumable-streaming-client` | 1 | M / Med | medium | MED | Claude Code |
| 6 | `mount-governance-guardrails` | 1 | L / High | frontier | HIGH | Claude Code |

## Execution rounds

- **Round 0 (gate):** merge prior-phase branches to `main`.
- **Round 1 (parallel):** `add-run-cancellation`, `resumable-streaming-client`, `mount-governance-guardrails`, `wire-sycophancy-detection` (HP1 first / isolated — HP4 shares the response path).
- **Round 2 (after HP1 orchestrator surface):** `emit-runtime-step-events`, then `wire-otlp-tracing-and-cost`.

## Deferred (out of phase)

- HP7 eval harness → `uar-safety-and-evals` phase
- Tool-approval Cedar migration → fold into `uar-safety-and-evals`
- Durable workflows / checkpointing → own phase
- Config write-back to YAML → own change
- Parking-lot `HookBus` → **will not build** (redundant with `RunEventEmitter`)
