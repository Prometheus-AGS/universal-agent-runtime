# Handoff in — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-pnpm-lock-submodule-consistency

**Spawned by:** perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## Why this child was spawned

The clean `screen-by-screen-validation` certification worktree rejected the
committed root lock with `ERR_PNPM_OUTDATED_LOCKFILE`. The pinned
`frontend/packages/prometheus-entity-management` manifest adds 17 importer
dependencies and changes 12 importer specifiers that the committed lock does
not contain. The parent execution contract requires a narrowly scoped
follow-up for this newly observed defect before certification can resume.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md

## Success criteria

- The root `pnpm-lock.yaml` matches every committed workspace manifest,
  including the pinned entity-management submodule.
- `pnpm install --lockfile-only --frozen-lockfile --ignore-scripts` exits 0
  without changing the lockfile hash.
- The child changes no manifest, dependency pin, product source, generated
  frontend bundle, or parent certification evidence.
- Strict OpenSpec validation, scoped integrity checks, and independent
  artifact review pass before the child returns control to the parent.

## Expected deliverables

- One reviewed root lockfile update.
- A strict-valid OpenSpec change with command/output evidence.
- Child assessment, plan, execution, reflection, scope, and handoff records.
- A clean source commit from which `screen-by-screen-validation` can restart
  preparation and browser certification.
