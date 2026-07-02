# Goals

Phase: **uar-next-harness**

Mandate: implement **every** recommendation in `docs/uar-next.md` (the
comprehensive architecture & production-readiness analysis, pulled to main as
commit `f58514e`) to make UAR the world's best agent harness. The doc's own
§17 implementation plan (5 sub-phases, 20 weeks) plus §18 Skill Pack
integration define the recommendation inventory; goals below group them.
`/kbd-assess` must verify each item against the current codebase (some may
already be partially done) and `/kbd-plan` must sequence them — expect nested
child phases (`/kbd-new-child`) per group given the scope.

## G1 — Foundation (doc §17 Phase 1, P0)

- Fix A2A gRPC transport (tonic-build v0.14 API changes; export gRPC module;
  transport functional end-to-end).
- Implement Postgres credential store (multi-tenant encryption parity with
  SurrealDB path).
- Add dynamic model router: `Router` module with capability registry, cost
  calculator, health monitor (doc §1.2 Gap 1, §4.3).
- Add prompt dialect engine: `PromptDialectEngine` with
  XML/JSON/Markdown/GLM transformations + dialect detection (doc §1.2 Gap 2,
  §12).
- Fix MemPalace (`rusqlite`/`libsqlite3-sys` version conflicts).
- Unpin dependencies (semver ranges for `surrealdb`, `pgvector`, etc.).
- Add Docker Compose full stack (UAR + Postgres + vector DB) for local
  evaluation.
- Resolve the critical gaps / problematic-code items in doc §14.

## G2 — Intelligence (doc §17 Phase 2, P1)

- Wire Cedar policy engine into the orchestrator tool loop (replace keyword
  heuristic with real Cedar evaluation for tool approval).
- Per-model context strategy: model-aware chunking, placement, compression
  (doc §1.2 Gap 3, §13.3).
- Cost tracking: per-model, per-task spend with budget alerts.
- Skill activation metrics: precision/recall per skill, per model (doc §8.3).
- Skill schema transformation to each model's native tool format.
- Extract RAG to a Knowledge Service (separate MCP service with query
  decomposition, verification, audit — doc §5.3; apply §5.4 mitigations if it
  stays in-process as an interim step).
- Provider health monitoring with automatic failover (doc §4.2).
- Model capability registry importing 2026 benchmark data (doc §4.2, §15).

## G3 — UX & Integration (doc §17 Phase 3, P2)

- Redesign web UI configuration (Cherry Studio-inspired provider config +
  unified settings page — doc §9.3, §10.3).
- Model comparison dashboard (benchmarks, costs, capabilities side-by-side).
- Real-time agent state visualization (reasoning steps, tool calls, context
  usage, routing decisions).
- Cost tracking dashboard (per-model, per-task, budget alerts).
- LibreFang integration: LibreFang tasks → UAR agents via A2A; UAR streams →
  LibreFang dashboard via AG-UI; shared model router (UAR liter-llm + router
  as LibreFang's LLM layer) — doc §2.2, §7.3, §16.
- Prometheus Skill Pack bundled with UAR distribution (doc §18: submodule,
  pre-compiled signed AgentDescriptors, lazy loading, version pinning,
  `uar skill update`, batch/incremental compilation, distribution channels).

## G4 — Specification & Distribution (doc §17 Phase 4, P2)

- Agent spec v2.0: add `model_requirements`, `prompt_dialect`,
  `rag_configuration`, `context_strategy`, `api_harness` (doc §3.2–3.3).
- Update PMPO compiler stages for the v2.0 fields.
- Conformance testing (agents run with declared requirements).
- Agent template library (coding, vision, terminal, etc.).
- Documentation overhaul (architecture, routing, dialects) + production
  deployment guide (Kubernetes, Docker, monitoring, scaling).
- Benchmark suite: automated tests for skill activation, routing accuracy,
  context efficiency.

## G5 — Polish & Release (doc §17 Phase 5, P3)

- Performance optimization of the hot path (router, dialect engine, context
  manager).
- Security audit (prompt-injection resistance, RBAC validation).
- Load testing (1000+ concurrent agents, failover under load).
- Documentation finalization; release candidate → GA.

## Success criteria

- Every table row in doc §17 and every §18 integration point is either
  implemented and verified, or explicitly dispositioned in the phase record
  (e.g. operator-only, needs-external-team, superseded) with rationale.
- The three headline changes from doc §19 — dynamic model routing, prompt
  dialect engine, A2A completion + LibreFang integration — are demonstrably
  working (tests + live smoke).
- Existing behavior preserved (Rule 32); all work lands as small reviewable
  PRs with tests (Rules 30/31).

## Carry-over inputs (from gate-activation-and-security-cleanup reflection)

These remain live and should be folded into assessment/planning:

- P0 operator: activate the eval gate (secret + seed baseline) — human-only.
- P0 hygiene: resolve the long-lived dirty working tree.
- P1: automate artifact-refiner QA gate (carried 3 phases).
- P1: finish H8 sandbox + MCP-status metric recorders.
- H3 emit-runtime-step-events — unbuilt planned change.
- Cedar `is_tool_allowed` mounted at HTTP only (overlaps G2 Cedar wiring).
