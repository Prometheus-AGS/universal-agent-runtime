# Handoff out — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-frontend-pnpm-lock-consistency

**Status:** DONE

## Deliverables

- Corrected nested lock: `frontend/pnpm-lock.yaml`
- Archived OpenSpec change:
  `openspec/changes/archive/2026-08-20-fix-frontend-pnpm-lock-consistency/`
- Synced canonical requirement:
  `openspec/specs/frontend-build-tooling/spec.md`
- Final verification and controls:
  `openspec/changes/archive/2026-08-20-fix-frontend-pnpm-lock-consistency/verification.md`
- Final artifact and immutable history:
  `.refiner/artifacts/fix-frontend-pnpm-lock-consistency/` and
  `.refiner/history/fix-frontend-pnpm-lock-consistency/2026-08-20_21-07-18Z/`
- Child reflection: `reflection.md`

## Goal completion

See `reflection.md`. Status: DONE.

## Unresolved items

- Lifecycle scripts were disabled for dependency installation. The child makes
  no install-script behavior claim.
- Parent browser certification, generated bundles, release checks, and external
  installation were not run in this child.

## Recommendations to the parent (perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion)

Resume the existing parent change:

```bash
/opsx:apply screen-by-screen-validation
```

Create the retained certification from this child's new immutable source
commit. Start from empty dependency directories, require frozen installation,
launch fresh processes, and regenerate every source fingerprint, report, video,
screenshot, and receipt. Do not reuse any pre-child certification artifact.
