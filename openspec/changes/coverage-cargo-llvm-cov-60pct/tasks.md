## 1. CI workflow
- [x] 1.1 Create `.github/workflows/coverage.yml` running `cargo-llvm-cov --lcov --output-path lcov.info` on every PR.
- [x] 1.2 Add `--fail-under-lines 60` for `cargo test --features server-full` and `cargo test --features minimal`. Implemented as a matrix job (`minimal`, `server-full`) using `cargo llvm-cov ... --lib --fail-under-lines 60` (llvm-cov's own runner, not a separate `cargo test` invocation — this is llvm-cov's standard usage).
- [x] 1.3 Install `taiki-e/install-action@v2` for `cargo-llvm-cov`.
- [x] 1.4 Upload to Codecov via `codecov/codecov-action@v4`; token from `CODECOV_TOKEN` secret (`fail_ci_if_error: false` so a missing/misconfigured token doesn't block CI independently of the coverage gate itself).

## 2. Frontend coverage
- [x] 2.1 Add `@vitest/coverage-v8` to `frontend/package.json` devDependencies.
- [x] 2.2 Update `vitest.config.ts` to enable coverage with the `v8` provider (60% thresholds for lines/statements/functions/branches).
- [x] 2.3 New `test:coverage` script (`vitest run --coverage`) wired into `.github/workflows/coverage.yml`'s `frontend-coverage` job (no separate `frontend-ci.yml` exists; `ci.yml`'s `frontend` job is untouched, coverage runs in the new workflow).

## 3. Coverage baseline + drift
- [ ] 3.1 Run the coverage job locally on `main`; record the per-feature numbers in `docs/coverage-baseline.md`. **DEFERRED to the phase's consolidated validation pass** (KBD implementation-first policy: static inspection + `cargo check` during implementation, full test/coverage runs in one consolidated pass once all planned changes land). `docs/coverage-baseline.md` exists with the threshold/mechanism documented and the per-file table explicitly marked pending this run.
- [x] 3.2 `tools/coverage-drift.sh` — print `current - baseline` per file; fail if any file drops > 5 points. Verified locally against synthetic lcov + baseline fixtures (pass, small-delta pass, >5pt-drop fail, no-baseline-entry no-op).
- [x] 3.3 Add the drift script to `.github/workflows/coverage.yml` as a follow-up step (`Check coverage drift vs baseline`, blocking — currently a no-op until 3.1 populates the baseline table, per the spec's documented phased-rollout scenario).

## 4. Cleanup
- [x] 4.1 Remove `.grcovrc` (cargo-llvm-cov supersedes it; the file was committed but never wired). Confirmed no live script/workflow referenced it (`grep -rln ".grcovrc" tools/ scripts/ .github/` → empty).
- [x] 4.2 Update `TESTING.md` (repo's testing doc; no `docs/testing.md` exists) to point to `cargo-llvm-cov` + the new coverage workflow; also fixed `test-config.yaml`'s inert `rust_coverage_tool: grcov` reference for consistency.

## 5. Verification
- [ ] 5.1 Run the new workflow on a sample PR; confirm the badge and the delta render correctly. **DEFERRED** — requires an actual PR run in GitHub Actions (external to this implementation pass); part of the phase's consolidated validation.
- [ ] 5.2 Confirm `--fail-under-lines 60` blocks a PR that drops coverage below the threshold. **DEFERRED** for the same reason as 5.1; the `--fail-under-lines` mechanism itself is cargo-llvm-cov's standard, well-tested flag — this task is about observing it fire in this repo's actual CI, not about the mechanism's existence.
