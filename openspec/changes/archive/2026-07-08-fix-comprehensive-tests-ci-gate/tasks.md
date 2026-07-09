## 1. Create test-config.yaml

- [x] 1.1 Create `test-config.yaml` at repo root with `environments`, `coverage` tool-selection, and per-language `threshold` blocks (`rust`, `typescript`, `playwright`) matching the structure `tools/check-coverage.mjs` actually parses.
- [x] 1.2 Set interim, clearly-labeled-as-unmeasured coverage thresholds low enough not to immediately fail given this project's confirmed-thin current coverage (~5.8% frontend unit-test file coverage per this phase's assessment) — not the abandoned spec's 90%+/95% aspirational figures.
- [x] 1.3 Confirm `tools/check-coverage.mjs`'s regex parser (`` `${section}:[\s\S]*?threshold:[\s\S]*?${key}:\s*(\d+)` ``) actually matches the YAML structure written, via a local dry run of the parser logic against the new file.

## 2. Verify locally

- [x] 2.1 Confirm `test -f test-config.yaml && test -f docker-compose.test.yaml && test -f Dockerfile.test` all pass locally (the exact check `comprehensive-tests.yml`'s Pre-flight job runs).
- [x] 2.2 Confirm `git status --short` shows only `test-config.yaml` as a new file — no accidental changes to `tools/test-all.sh` or the workflow YAMLs (design.md's finding: neither needs changes).

## 3. Verify on real CI

- [x] 3.1 Push this change and dispatch `comprehensive-tests.yml` via `gh workflow run` (don't wait for its daily 6 AM UTC schedule).
- [x] 3.2 Confirm the Pre-flight job passes and every downstream job is dispatched (not skipped) — record which jobs pass/fail for real in `findings.md`, since none have ever run before.
- [x] 3.3 Dispatch `tests-full.yml` similarly and record its real outcome.

## 4. Findings

- [x] 4.1 Write `findings.md` documenting: the real per-job outcomes from both workflows' first-ever full dispatch, any newly-surfaced failures (expected, given they've never run), and confirmation that `tools/test-all.sh` needed no changes.
