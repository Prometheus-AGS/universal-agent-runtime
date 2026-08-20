# Handoff in — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-frontend-pnpm-lock-consistency

**Spawned by:** perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## Why this child was spawned

The parent screen certification stopped before browser execution because the
first clean frontend dependency preparation rejected and rewrote the committed
nested lock. The prior root-lock child covered only the repository-root pnpm
workspace; `frontend/pnpm-workspace.yaml` is a separate execution root with its
own authoritative lock.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md

## Success criteria

- pnpm 11.15.0 accepts the committed nested lock in frozen metadata-only mode.
- An empty-dependency-tree frozen install accepts the committed nested lock and
  leaves it byte-identical.
- Every HEAD-to-candidate lock mutation is causally tied to the current frontend
  manifests or pinned entity-management manifests; unrelated common snapshot
  resolutions remain unchanged.
- Parent TypeScript, lint, and focused SSE adapter checks pass without changing
  either the nested or root lock.
- The child OpenSpec change, evidence, artifact-refiner gate, and independent
  critic/judge review pass before control returns to screen certification.

## Expected deliverables

- Corrected `frontend/pnpm-lock.yaml` only; no manifest, source, root-lock, or
  submodule-pin change.
- Archived OpenSpec change and synced `frontend-build-tooling` requirement.
- Replayable negative-control, delta-audit, clean-install, focused-test, and
  artifact-integrity evidence.
- A child commit and handoff naming the exact screen-certification resume point.
