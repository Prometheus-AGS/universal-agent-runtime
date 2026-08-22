## 1. Baseline and Candidate Selection

- [x] 1.1 Retain the clean `fa4ffb96` frozen-install negative control with its non-zero exit, `ERR_PNPM_OUTDATED_LOCKFILE`, 17 added dependencies, and 12 mismatched specifiers.
- [x] 1.2 Record the initial operator candidate and twice-replayed clean regeneration, then audit the corrected candidate directly against HEAD to preserve the two noncausal `minimatch` and `y-webrtc` edges while retaining the new direct `ws` pin.

## 2. Root Lock Repair

- [x] 2.1 Adopt and minimally correct only the operator-owned root `pnpm-lock.yaml` candidate, verify its SHA-256 is `645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`, and confirm no manifest, submodule pin, product source, generated bundle, or parent certification file is included.
- [x] 2.2 Run frozen lock-only and full frozen installation with lifecycle scripts disabled, recording exit 0 and identical pre/post lock digests for both commands.
- [x] 2.3 Run `pnpm typecheck` and `pnpm lint` as the TypeScript Tier 0 checks and record their actual results without expanding into warning cleanup.

## 3. Verification and Independent Review

- [x] 3.1 Verify the entity-management Git link remains `0352c83d7b386db56ffea8304ffdf3e2edb00fc8`, the scoped diff contains only permitted child files, and `git diff --check` exits 0.
- [x] 3.2 Run `openspec validate fix-pnpm-lock-submodule-consistency --strict --no-interactive` and write `verification.md` with per-requirement results, exact commands and outputs, the stale-lock negative control, profile limits, and no aggregate verdict.
- [x] 3.3 Produce and validate the artifact-refiner snapshot, then obtain history-free critic and judge approval of the lock, contract, scope, and evidence before archive.

## 4. Sync, Reflect, and Parent Handoff

- [x] 4.1 Sync and archive the approved delta into `frontend-build-tooling`, then strict-validate the canonical spec and the archived change.
- [x] 4.2 Complete the child reflection and canonical child exit, write exact deliverables and remaining limits to `handoff-out.md`, and restore parent work to `/opsx:apply screen-by-screen-validation` without changing the 70/79 outer denominator.
- [x] 4.3 Append the observed stale-lock and range-drift lesson to `.prometheus`, stage only the permitted child lock/spec/KBD/refiner/history artifacts, and create one source commit without staging operator settings, generated static output, prior certification bundles, or unrelated lockfiles.
