## Context

See proposal.md for the observed gap. This is a Spec-stage design boundary, not an implemented result. The phase analysis selects reuse of the existing host and pins; the following Plan prerequisites remain binding.

scope: `src/uar/rag/`, `src/uar/runtime/prompt/`, `src/uar/runtime/manager.rs`, `tests/bdd/steps/kb-retrieval.steps.ts`, `tests/bdd/features/chat-kb-retrieval.feature` and scenario fixtures under existing test roots. Plan must enumerate exact edit files before Execute. Shared files are edited in the phase index's total order; no concurrent writer to manager, orchestrator, prompt assembly or a build target.

## Goals / Non-Goals

Goals: satisfy this change's spec scenarios through existing governed production paths with local evidence.

Non-goals: replacement agent frameworks, dependency upgrades, UI redesign, kernel mutations, deployment or release. No product-code edit is performed by this Spec stage.

## Decisions

1. Reuse the existing retrieval/decomposition/dedup/verification pipeline and provenance events; keep ingestion and hybrid/embedding integrations intact. Link answer claims to authorized versioned source evidence.

2. Place access checks at retrieval and actual use, including retry/resume and user-visible audit. Mark derivatives of revoked sources invalid; preservation cannot grant access.

3. Distinguish support, insufficiency, conflict and staleness with a frozen evidence oracle. Lexical matching is a diagnostic, not a factual verification algorithm. Optional model judging cannot bypass deterministic access/integrity gates.

4. Retain the current UI/domain-hook/entity architecture. This change adds runtime evidence contracts and tests, not a new knowledge dashboard.

## Required planning prerequisites

PLANNING BLOCKED for verification algorithm/library commitments until targeted external RAG/evidence-verification candidates are compared with existing seams and a dataset/verdict acceptance oracle is frozen. Document provenance, license/pin fit and adopt/adapt/build rationale; settle verification mechanisms before executable tasks.

Implementation follows harness-skill-activation-quality.

## Risks / Trade-offs

The uncomfortable scenario: A correct citation can accompany an unsupported claim → require claim/evidence verdict fixtures. Revocation can leak through summaries or caches → invalidate derivatives and capture outbound requests across retry/restart.

## Migration Plan

Add verdict/provenance data compatibly to existing records/events. Historical lexical matches remain unverified until evaluated. Rollback preserves authorization boundaries and does not relabel unverified historical content as supported. All rollout and rollback steps are future implementation work with local validation; this artifact authorizes no deployment.

## Verification contract

Map every scenario to an executable fixture during Plan. Run Tier 0 per edit, Tier 1 per completed unit and Tier 2 at phase completion. Tier 3 belongs only to the separately registered local certification milestone. Request-level fixtures and production-path traces are required where specified; module presence and build success are not behavior proof. Artifact validation in this stage cannot establish runtime correctness.


## Plan resolution (2026-09-16)

The historical Spec-stage prerequisite list above is retained. Its concrete resolution and remaining execution gates are in `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`, `plan-research.md`, `execution-map.json`, `acceptance-contract.json`, `provider-profile-matrix.json` and `profile-certification.md` (repository-relative paths). The Plan manifest records current hashes; the earlier Spec manifest remains a historical snapshot. No implementation checkbox is completed by these planning artifacts.

- library: cand-001 — Existing UAR typed prompt/turn, routing, persistence and governed evaluation seams; verdict: adapt. Preserve capability inversion and established host integration; implement missing contracts rather than replacing runtime.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/prompt/assemble.rs — Inspected local source renders typed fragments with two family preference booleans; extend existing seam.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/context/reduce.rs — Inspected reducer can drop severed tool groups; new protected preservation requires a stronger contract.
- library: cand-010 — Ragas faithfulness / claim decomposition; verdict: reference. Reference claim-level support; extend UAR verification with governed Liter extraction/entailment plus deterministic source/version/authorization checks. Model judgments remain labeled advisory; malformed/unknown evidence abstains.
  Evidence: https://docs.ragas.io/en/stable/concepts/metrics/available_metrics/faithfulness/ — Faithfulness decomposes an answer into claims and assesses support in supplied context; it is not an access-control mechanism.

Exact owned files, proposed fixtures, commands and per-task evidence directories are in execution-map.json. The reviewed order serializes shared files. No product behavior edit precedes its frozen baseline; no actual model profile is enabled before endpoint/count/output evidence; no phase completion precedes the separately registered local certification milestone.
