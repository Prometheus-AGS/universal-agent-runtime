## Context

See proposal.md for the observed gap. This is a Spec-stage design boundary, not an implemented result. The phase analysis selects reuse of the existing host and pins; the following Plan prerequisites remain binding.

scope: `src/uar/runtime/skills/`, `src/uar/runtime/native_skills/activate_skill.rs`, `src/uar/runtime/prompt/`, `tests/skill_activation_runtime.rs` and scenario fixtures under existing test roots. Plan must enumerate exact edit files before Execute. Shared files are edited in the phase index's total order; no concurrent writer to manager, orchestrator, prompt assembly or a build target.

## Goals / Non-Goals

Goals: satisfy this change's spec scenarios through existing governed production paths with local evidence.

Non-goals: replacement agent frameworks, dependency upgrades, UI redesign, kernel mutations, deployment or release. No product-code edit is performed by this Spec stage.

## Decisions

1. Extend current catalog/activation/provenance services and preserve conversation > agent > global scope resolution. Explicit governed selection remains authoritative.

2. Keep immutable active skill version/body in canonical state and reattach it before final budget validation. Reclaiming a projection is permitted only if canonical data survives; post-count injection and silently missing active skills are rejected.

3. Retain the existing exhaustive oracle and Recall@10 >=99% candidate gate. Measure task success separately from tool attribution, including prompt-only skills.

4. Reuse early deterministic fixtures and the shared preparation contract. Ranking methodology remains a required researched Plan choice; no new library has been selected.

## Required planning prerequisites

PLANNING BLOCKED for activation evaluation implementation commitments until targeted external skill-selection/evaluation candidates are compared with current services. Freeze labeled datasets, baseline and numerical precision/recall/unnecessary-activation/task-success/latency/token-overhead thresholds before implementation. These are prerequisite work, not post-hoc acceptance choices.

Implementation follows harness-task-graph-lifecycle.

## Risks / Trade-offs

The uncomfortable scenario: High retrieval recall can coexist with harmful activations → include negative/ambiguous cases and task outcomes. Active skill bodies may exceed model allowance → explicit overflow rather than silent removal.

## Migration Plan

Preserve explicit activation and legacy-mode compatibility. Run candidate selection in shadow until existing recall and new frozen quality gates pass. Rollback ranking independently while keeping immutable body preservation and authorization checks. All rollout and rollback steps are future implementation work with local validation; this artifact authorizes no deployment.

## Verification contract

Map every scenario to an executable fixture during Plan. Run Tier 0 per edit, Tier 1 per completed unit and Tier 2 at phase completion. Tier 3 belongs only to the separately registered local certification milestone. Request-level fixtures and production-path traces are required where specified; module presence and build success are not behavior proof. Artifact validation in this stage cannot establish runtime correctness.


## Plan resolution (2026-09-16)

The historical Spec-stage prerequisite list above is retained. Its concrete resolution and remaining execution gates are in `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`, `plan-research.md`, `execution-map.json`, `acceptance-contract.json`, `provider-profile-matrix.json` and `profile-certification.md` (repository-relative paths). The Plan manifest records current hashes; the earlier Spec manifest remains a historical snapshot. No implementation checkbox is completed by these planning artifacts.

- library: cand-001 — Existing UAR typed prompt/turn, routing, persistence and governed evaluation seams; verdict: adapt. Preserve capability inversion and established host integration; implement missing contracts rather than replacing runtime.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/prompt/assemble.rs — Inspected local source renders typed fragments with two family preference booleans; extend existing seam.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/context/reduce.rs — Inspected reducer can drop severed tool groups; new protected preservation requires a stronger contract.
- library: cand-009 — Anthropic skill-creator evaluation methodology; verdict: reference. Reference held-out trigger/outcome methodology; adapt current skill activation fixtures and exhaustive oracle. No upstream scripts are copied or executed.
  Evidence: https://github.com/anthropics/skills/blob/main/skills/skill-creator/SKILL.md — The source separates trigger positives/near-miss negatives and outcome assertions, tracks timing/token comparisons, and separates training from held-out cases.
  Evidence: https://raw.githubusercontent.com/anthropics/skills/main/skills/skill-creator/LICENSE.txt — Skill-local license is Apache-2.0; repository root metadata alone reported no license.

Exact owned files, proposed fixtures, commands and per-task evidence directories are in execution-map.json. The reviewed order serializes shared files. No product behavior edit precedes its frozen baseline; no actual model profile is enabled before endpoint/count/output evidence; no phase completion precedes the separately registered local certification milestone.
