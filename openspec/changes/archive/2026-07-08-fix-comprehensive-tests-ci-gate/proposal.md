## Why

`.github/workflows/comprehensive-tests.yml` and `.github/workflows/tests-full.yml` have never passed their first "Pre-flight Checks"/"Checking Prerequisites" step since this repository's initial commit (2026-01-19) — both require `test-config.yaml`, which has never existed. Every downstream job (Code Quality, Security Audit, Build Verification, Docker Integration Tests, Comprehensive Tests, Performance Benchmarks) has been unconditionally skipped, on every run, forever. This traces to an abandoned Spec Kit feature (`specs/001-testing-infrastructure/`, 0/74 tasks complete) whose partial dead code (`src/testing/`) was already deleted in a prior phase without cleaning up the two workflows still referencing its never-built config file. A prior tool assessment (`docs/CODEX_ASSESSMENT.md`, 2025-12-31) already diagnosed this exact issue and proposed the fix; it was never applied.

## What Changes

- Create a real `test-config.yaml` at the repo root: test-runner configuration (environments, coverage-tool selection, per-language coverage thresholds) matching the structure already documented in `specs/001-testing-infrastructure/quickstart.md` and already consumed by `tools/check-coverage.mjs` (which reads `TEST_CONFIG_FILE`, defaulting to `test-config.yaml`, for per-language `threshold` blocks).
- No code changes to `tools/test-all.sh` — it already correctly separates `CONFIG_FILE` (server config, `config.test.yaml`) from `TEST_CONFIG_FILE` (test-runner config, `test-config.yaml`) as two distinct env vars. The Codex assessment's config-drift concern is already resolved in the current script; only the file itself was missing.
- Coverage thresholds are set to an honest interim baseline reflecting this project's actual current coverage posture (confirmed thin — ~5.8% frontend unit-test file coverage per this phase's own assessment), not the aspirational 90%+/95% figures in the abandoned spec, which would just move the failure point rather than let the pipeline complete.

## Capabilities

### New Capabilities

- `comprehensive-test-execution`: covers the repo's CI pre-flight config-validation gate and coverage-threshold enforcement for the "comprehensive"/"full" test workflows — previously undocumented as a capability since it never actually executed.

### Modified Capabilities

(none)

## Impact

- New file: `test-config.yaml` (repo root).
- No changes to `.github/workflows/comprehensive-tests.yml`, `.github/workflows/tests-full.yml`, or `tools/test-all.sh` — the file's mere existence unblocks the Pre-flight gate; script logic is already correct.
- Verified by dispatching both workflows for real on GitHub Actions and confirming they progress past Pre-flight (per this project's established `CI Trigger Actually Fires` requirement).
- KBD workflow state: belongs to phase `uar-production-ready-uiux-2026-07`; updated via `/kbd-apply`.
