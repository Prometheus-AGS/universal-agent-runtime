# Handoff out — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-pnpm-lock-submodule-consistency

**Status:** DONE

## Deliverables

- Corrected root lock: `pnpm-lock.yaml`
- Archived OpenSpec change:
  `openspec/changes/archive/2026-08-20-fix-pnpm-lock-submodule-consistency/`
- Synced canonical requirement:
  `openspec/specs/frontend-build-tooling/spec.md`
- Final verification and failed controls:
  `openspec/changes/archive/2026-08-20-fix-pnpm-lock-submodule-consistency/verification.md`
- Final artifact and immutable history:
  `.refiner/artifacts/fix-pnpm-lock-submodule-consistency/` and
  `.refiner/history/fix-pnpm-lock-submodule-consistency/2026-08-20_17-25-32Z/`
- Child reflection: `reflection.md`

## Goal completion

See reflection.md. Status: DONE.

## Unresolved items

- Lifecycle scripts were disabled for dependency installation. The child makes
  no install-script behavior claim.
- Parent browser certification, generated bundles, release checks, and external
  installation were not run in this child.
- The KBD control plane at `127.0.0.1:7892` was unreachable; canonical commands
  committed locally and restored the original outer `70/79` denominator.

## Recommendations to the parent (perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion)

Resume the existing parent change:

```bash
/opsx:apply screen-by-screen-validation
```

Create the retained certification from this child's new immutable source
commit. Start from empty dependency directories, require frozen installation,
launch fresh processes, and regenerate every source fingerprint, report, video,
screenshot, and receipt. Do not reuse any pre-child certification artifact.
