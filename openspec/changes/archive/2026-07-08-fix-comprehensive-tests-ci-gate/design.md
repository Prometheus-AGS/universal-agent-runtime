## Context

`comprehensive-tests.yml`'s Pre-flight job runs `test -f test-config.yaml || exit 1`, unconditionally skipping every downstream job when it fails — which it always has. `tools/check-coverage.mjs` (a real, already-wired script, called from `test-all.sh`'s `check_coverage_thresholds` step) reads `TEST_CONFIG_FILE` (defaulting to `test-config.yaml`) and parses per-language `threshold` blocks via a regex: `` `${section}:[\s\S]*?threshold:[\s\S]*?${key}:\s*(\d+)` `` for sections `rust`, `typescript`, `playwright` and keys `line`/`function`/`branch`. If the file is missing, it silently falls back to hardcoded defaults (`rust: 90/85/80`, `typescript: 85/80/75`, `e2e: 70/65`) — aspirational figures from the abandoned `specs/001-testing-infrastructure` spec, not measured against this codebase's actual coverage.

## Goals / Non-Goals

**Goals:**
- Get `comprehensive-tests.yml` and `tests-full.yml` past Pre-flight on a real, observed GitHub Actions run.
- Set coverage thresholds that reflect reality, not the abandoned spec's aspiration — so the pipeline can actually complete (or fail meaningfully further downstream on gaps that are real, not on gaps that are cosmetic file-existence bugs).

**Non-Goals:**
- Actually raising this project's test coverage to any particular percentage — that's Goal 3 of this phase generally, not this one narrow CI-gate fix. This change unblocks measurement; it doesn't do the measuring-and-improving work itself.
- Rebuilding any part of the abandoned `specs/001-testing-infrastructure` feature (custom `TestSuite`/`TestCase`/`QualityGate` entities, certification suites, flaky-test detection). Out of scope — that spec's ambition was already identified as a cautionary precedent in this phase's own assessment.
- Modifying `tools/test-all.sh` or the workflow YAML — both already correctly separate `CONFIG_FILE` from `TEST_CONFIG_FILE`; nothing there is broken.

## Decisions

**1. Set coverage thresholds to an honest, low interim baseline rather than copying the abandoned spec's 90%/95% aspirational figures.**
Alternative considered: copy `quickstart.md`'s documented `95.0`/`98.0` thresholds verbatim, since they're already written down. Rejected — this project's own assessment (this phase, 2026-07-08) found frontend unit-test coverage at ~5.8% (12/206 files). Setting a 90%+ bar today would just relocate the permanent-failure point from "Pre-flight" to "Coverage Threshold Checks," which is not meaningfully different from the current broken state — it would still never go green, just later in the pipeline. Chosen instead: low, clearly-labeled interim thresholds (documented in the file's own comments as "interim baseline, ratchet up as coverage work lands," mirroring this project's established eval-harness pattern of baseline-and-ratchet rather than aspirational-and-perpetually-red).

**2. Don't measure exact current coverage before setting the interim numbers.**
Getting an exact Rust/TypeScript/Playwright coverage percentage would require running `grcov`/`cargo-llvm-cov` and Playwright's V8 coverage collection — both non-trivial to set up correctly in one change, and arguably their own scope (part of "close test coverage gaps," not "fix the CI gate"). Chosen: conservative, clearly-disclosed placeholder thresholds low enough to not immediately fail, with an explicit comment flagging them as unmeasured placeholders pending real coverage tooling — honest about the limitation rather than pretending precision that wasn't verified.

**3. Introduce a new `comprehensive-test-execution` capability rather than force-fitting this into an existing one.**
No existing `openspec/specs/` capability covers CI test-execution infrastructure specifically (closest candidates — `frontend-validation-gate`, `eval-harness` — are about different concerns). A new capability accurately scopes future requirements about this specific CI surface.

## Risks / Trade-offs

- **[Risk]** Even with `test-config.yaml` created, `comprehensive-tests.yml`'s later jobs (Docker Integration Tests, Performance Benchmarks) may fail for unrelated reasons never previously observed, since they've never run. → **Mitigation**: this change's verification requirement is "progress past Pre-flight," not "every job goes green" — if later jobs fail, that's real, newly-surfaced information this fix exists to produce, not a regression this change introduces. Document what's found in `findings.md` regardless of outcome.
- **[Risk]** The low interim coverage thresholds could be mistaken for "coverage is fine" if not clearly labeled. → **Mitigation**: inline YAML comments explicitly state these are unmeasured interim placeholders, not a real assessment of coverage health.

## Migration Plan

Additive only — one new file, no changes to existing workflow/script logic. No rollback complexity.

## Open Questions

None blocking.
