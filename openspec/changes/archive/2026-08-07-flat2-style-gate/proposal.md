## Why

The frontend has 630 existing border idiom matches plus legacy outline, shadow, blur, gradient, and non-kebab-case filenames. Removing all of that debt belongs to later migration changes, but without a mechanical gate new violations can accumulate while those changes proceed. C-03 freezes the current baseline and makes any increase fail immediately.

## What Changes

- Add the approved Flat 2.0 `no-restricted-syntax` selectors to the frontend ESLint configuration.
- Add `eslint-plugin-unicorn` and enforce `unicorn/filename-case` with kebab-case as the forward filename contract.
- Record current violations in an exact, shrinking allowlist and add a deterministic checker that fails on either new findings or stale allowlist entries.
- Apply narrow ESLint overrides only to files represented in the legacy allowlist; the standalone checker continues to inspect those files with the unsuppressed rules.
- Add negative fixtures and wire the style gate into the repository's existing CI grep-gate harness.
- Exclude generated coverage, test-result, and dedicated gate-fixture output from the product lint traversal.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-design-system`: Adds enforceable Flat 2.0 syntax and filename invariants with a shrinking legacy baseline.

## Impact

- **Runtime UX:** No component or runtime behavior changes; this change only freezes the current visual-debt baseline.
- **Dependencies/build:** Adds `eslint-plugin-unicorn@73.0.0`, compatible with the repository's Node 24 and ESLint 10.7 toolchain.
- **CI:** The root grep-gate script gains an exact Flat 2.0 no-regression check and negative-fixture proof.
- **Migration ownership:** C-05 and C-14a shrink style entries; later filename migration shrinks filename entries. C-03 does not perform those edits.
- **KBD workflow state:** C-03 start and completion are recorded through canonical KBD change transitions.
