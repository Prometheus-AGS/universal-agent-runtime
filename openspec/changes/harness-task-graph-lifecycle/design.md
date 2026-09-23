## Context

See proposal.md for the observed gap. This is a Spec-stage design boundary, not an implemented result. The phase analysis selects reuse of the existing host and pins; the following Plan prerequisites remain binding.

scope: `src/uar/api/a2a/`, `src/uar/persistence/agent_threads.rs`, `src/uar/runtime/manager.rs`, `src/uar/runtime/graph/`, `src/llm/orchestrator.rs` and scenario fixtures under existing test roots. Plan must enumerate exact edit files before Execute. Shared files are edited in the phase index's total order; no concurrent writer to manager, orchestrator, prompt assembly or a build target.

## Goals / Non-Goals

Goals: satisfy this change's spec scenarios through existing governed production paths with local evidence.

Non-goals: replacement agent frameworks, dependency upgrades, UI redesign, kernel mutations, deployment or release. No product-code edit is performed by this Spec stage.

## Decisions

1. Extend the existing host thread service and persisted agent-thread identity, not a parallel protocol scheduler. A2A and AG-UI remain adapters over authoritative committed host state.

2. Define owner-scoped durable identity and recovery state spanning creation/enqueue/publication. The storage atomicity and reconciliation mechanism is a required Plan design decision after targeted research, not a promise of external exactly-once effects.

3. Reuse existing RuntimeStep/event projections with correlation for graph/node/iteration/tool/child. Persist terminal truth and replay cursor identity; avoid content-bearing telemetry.

4. Keep approvals, budgets and cancellation in trusted hosts; children receive narrowing policy intersections. Unknown remote completion is inspectable reconciliation work, not clean cancellation.

5. Serialize shared manager/orchestrator edits after the context and model changes. Use their request-preparation and receipt contracts from graph/child paths.

## Required planning prerequisites

PLANNING BLOCKED for implementation commitments until two targeted external-candidate evaluations are recorded: durable A2A lifecycle/recovery, and graph/agent observability. Compare existing seams and external candidates, version/pin fit, migration cost and licensing; record adopt/adapt/build rationale and source evidence. Then settle crash atomicity, recovery and event ordering mechanisms before executable planning.

Implementation follows harness-model-profiles-routing.

## Risks / Trade-offs

The uncomfortable scenario: Persisting task identity without enqueue recovery strands work → crash fixtures at every boundary. Events can look complete while remote work lives → unresolved cleanup remains visible and blocks a clean terminal claim.

## Migration Plan

Version durable identity/event additions compatibly and backfill only unambiguous mappings. Keep ambiguous legacy mappings blocked for recovery. Preserve existing protocol field semantics; rollback must not spawn replacement runs for already accepted identities. All rollout and rollback steps are future implementation work with local validation; this artifact authorizes no deployment.

## Verification contract

Map every scenario to an executable fixture during Plan. Run Tier 0 per edit, Tier 1 per completed unit and Tier 2 at phase completion. Tier 3 belongs only to the separately registered local certification milestone. Request-level fixtures and production-path traces are required where specified; module presence and build success are not behavior proof. Artifact validation in this stage cannot establish runtime correctness.


## Plan resolution (2026-09-16)

The historical Spec-stage prerequisite list above is retained. Its concrete resolution and remaining execution gates are in `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`, `plan-research.md`, `execution-map.json`, `acceptance-contract.json`, `provider-profile-matrix.json` and `profile-certification.md` (repository-relative paths). The Plan manifest records current hashes; the earlier Spec manifest remains a historical snapshot. No implementation checkbox is completed by these planning artifacts.

- library: cand-001 — Existing UAR typed prompt/turn, routing, persistence and governed evaluation seams; verdict: adapt. Preserve capability inversion and established host integration; implement missing contracts rather than replacing runtime.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/prompt/assemble.rs — Inspected local source renders typed fragments with two family preference booleans; extend existing seam.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/context/reduce.rs — Inspected reducer can drop severed tool groups; new protected preservation requires a stronger contract.
- library: cand-007 — A2A Python SDK TaskStore / AgentExecutor; verdict: reference. Reference lifecycle separation; adapt cand-001 persistence/thread host with durable correlation and acceptance intent. Do not install the Python SDK.
  Evidence: https://github.com/a2aproject/a2a-python/blob/main/src/a2a/server/agent_execution/agent_executor.py — The official executor separates execution/cancellation from event publication; TaskStore is a distinct server concern.
- library: cand-008 — OpenTelemetry GenAI agent/tool conventions; verdict: reference. Keep existing RuntimeStep and tracing/OTLP dependencies. Add host correlation fields and optional convention mapping; no new exporter or observability service.
  Evidence: https://opentelemetry.io/docs/specs/semconv/registry/attributes/gen-ai/ — The attribute registry provides tool identity conventions; the former agent-span page now redirects readers to the dedicated GenAI repository.

Exact owned files, proposed fixtures, commands and per-task evidence directories are in execution-map.json. The reviewed order serializes shared files. No product behavior edit precedes its frozen baseline; no actual model profile is enabled before endpoint/count/output evidence; no phase completion precedes the separately registered local certification milestone.
