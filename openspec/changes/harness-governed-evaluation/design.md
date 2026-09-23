## Context

See proposal.md for the observed gap. This is a Spec-stage design boundary, not an implemented result. The phase analysis selects reuse of the existing host and pins; the following Plan prerequisites remain binding.

scope: `src/uar/eval/`, `tests/`, `evals/`, `frontend/` and scenario fixtures under existing test roots. Plan must enumerate exact edit files before Execute. Shared files are edited in the phase index's total order; no concurrent writer to manager, orchestrator, prompt assembly or a build target.

## Goals / Non-Goals

Goals: satisfy this change's spec scenarios through existing governed production paths with local evidence.

Non-goals: replacement agent frameworks, dependency upgrades, UI redesign, kernel mutations, deployment or release. No product-code edit is performed by this Spec stage.

## Decisions

1. Extend the current runner with a governed host target; keep completion suites and advisory judges. Use deterministic local tools and recorded providers to exercise actual approvals, graph paths, budgets and receipts.

2. Each earlier change owns its early regression fixtures. This final change integrates them into a strict baseline/delta gate; it does not delay context correctness tests until the end.

3. Replace existing eval CI requirements with local runs. Inventory workflows and invoked scripts during Plan; change only observed violations and run pnpm github-actions-policy:validate before and after any workflow/invoked-script edit. No general product test belongs in GitHub Actions.

4. Capture revision/profile/environment, baseline identity, metric denominator/exclusions and raw results. Historical failure/coverage numbers remain claims until reproduced. Keep the existing 60% coverage threshold.

5. Register a separate local certification milestone during Plan for Tier 3 profile evidence. It is a prerequisite of phase completion; registration is not certification and implies neither release nor deployment.

## Required planning prerequisites

PLANNING BLOCKED for agentic-evaluation framework/algorithm commitments until targeted external candidates are compared with the existing eval runner. Plan must identify the two reported failure cases, exact coverage metric/exclusions, supported-profile matrix, live requirements, seeded faults and frozen baseline thresholds. Missing required evidence blocks completion.

Implementation follows harness-knowledge-evidence.

## Risks / Trade-offs

The uncomfortable scenario: A test adapter may bypass production governance → capture real host traces and seeded deny/receipt failures. Missing hardware or live endpoints can stall certification → preserve environment-blocked rows rather than lower scope. Broad coverage work may expose unrelated UI defects → scope behavior tests; any necessary UI code change requires its own routing/design review before execution.

## Migration Plan

Keep completion suite deserialization and CLI behavior. Add governed suites/baselines explicitly; do not auto-update baselines in certification. Migrate obsolete workflow testing to local invocations under the policy validator. Rollback keeps deterministic/local evidence and never reinstates forbidden Actions tests. All rollout and rollback steps are future implementation work with local validation; this artifact authorizes no deployment.

## Verification contract

Map every scenario to an executable fixture during Plan. Run Tier 0 per edit, Tier 1 per completed unit and Tier 2 at phase completion. Tier 3 belongs only to the separately registered local certification milestone. Request-level fixtures and production-path traces are required where specified; module presence and build success are not behavior proof. Artifact validation in this stage cannot establish runtime correctness.


## Plan resolution (2026-09-16)

The historical Spec-stage prerequisite list above is retained. Its concrete resolution and remaining execution gates are in `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`, `plan-research.md`, `execution-map.json`, `acceptance-contract.json`, `provider-profile-matrix.json` and `profile-certification.md` (repository-relative paths). The Plan manifest records current hashes; the earlier Spec manifest remains a historical snapshot. No implementation checkbox is completed by these planning artifacts.

- library: cand-001 — Existing UAR typed prompt/turn, routing, persistence and governed evaluation seams; verdict: adapt. Preserve capability inversion and established host integration; implement missing contracts rather than replacing runtime.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/prompt/assemble.rs — Inspected local source renders typed fragments with two family preference booleans; extend existing seam.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/context/reduce.rs — Inspected reducer can drop severed tool groups; new protected preservation requires a stronger contract.
- library: cand-011 — Inspect AI external-agent evaluation; verdict: reference. Reference solver/scorer/log separation; adapt existing Rust eval runner with a governed target and deterministic host fixtures. External Inspect integration is unnecessary for these exit criteria.
  Evidence: https://github.com/UKGovernmentBEIS/inspect_ai/blob/main/src/inspect_ai/_eval/task/task.py — Task construction accepts custom solver/agent and scorers; evaluation exposes repeated epochs and retained logs.

Exact owned files, proposed fixtures, commands and per-task evidence directories are in execution-map.json. The reviewed order serializes shared files. No product behavior edit precedes its frozen baseline; no actual model profile is enabled before endpoint/count/output evidence; no phase completion precedes the separately registered local certification milestone.
