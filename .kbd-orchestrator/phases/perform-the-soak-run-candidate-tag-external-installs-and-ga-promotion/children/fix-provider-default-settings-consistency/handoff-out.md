# Handoff out — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-provider-default-settings-consistency

**Status:** DONE

## Deliverables

- Product repair:
  - `src/uar/settings/manager.rs`
  - `src/uar/api/providers.rs`
  - `tests/settings_persistence.rs`
- Archived OpenSpec change and synced capability:
  - `openspec/changes/archive/2026-08-19-fix-provider-default-settings-consistency/`
  - `openspec/specs/provider-model-settings-certification/spec.md`
- Verification and independent review:
  - `openspec/changes/archive/2026-08-19-fix-provider-default-settings-consistency/verification.md`
  - `.refiner/artifacts/fix-provider-default-settings-consistency/`
  - `.refiner/history/fix-provider-default-settings-consistency/2026-08-19_23-13-23Z/`
- Child reflection: `reflection.md`

## Goal completion

See reflection.md. Status: DONE.

## Unresolved items

- Registry-only deployments without a configured settings manager intentionally retain no durability guarantee.
- Concurrent provider deletion between pre-validation and publication remains outside this child's cross-store consistency boundary.
- Parent Providers/Auth/MCP browser checks and full screen recertification were not run in the child.
- The canonical KBD control plane at `127.0.0.1:7892` was unreachable; runtime commands committed locally and preserved the original outer implementation denominator at `70/79` while recording this child separately at `1/1`.

## Recommendations to the parent (perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion)

Resume the existing parent change without rewriting its evidence:

```bash
/opsx:apply screen-by-screen-validation
```

Run the already-authored focused browser check before full recertification:

```bash
CI=1 pnpm exec playwright test -c tests/bdd/playwright.config.ts tests/bdd/.features-gen/features/product-screen-validation.feature.spec.js --grep 'Providers changes|Auth mints|MCP health'
```

Do not rerun the child Cargo checks unless one of the three product files changes. Continue the parent and remaining original release work toward `79/79`.
