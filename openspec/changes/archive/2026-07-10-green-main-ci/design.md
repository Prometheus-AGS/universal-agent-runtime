## Context
See proposal; failures enumerated in the phase assessment (§3). The ci.yml
fix predates this phase as an uncommitted working-tree diff with inline
rationale; review confirmed it is sound and matches the project's
`cargo clippy --lib` baseline-tracking practice.

## Goals / Non-Goals
**Goals:** every workflow green or explicitly advisory; zero silent skips.
**Non-Goals:** fixing the ~500 pre-existing pedantic warnings (tracked
policy-wise by Cargo.toml lint levels); repairing the broken `model-build` /
`memory-palace` / `sandbox-microsandbox` features (disclosed in ci.yml
comments; separate work).

## Decisions
- D1: adopt the working-tree ci.yml diff as-is (feature scoping + no
  `-D warnings`), preserving its disclosure comments.
- D2 (REVISED at apply time — consolidation over resurrection): the four
  overlapping legacy test workflows (tests-quick.yml, tests-full.yml,
  quick-tests.yml, comprehensive-tests.yml) are DELETED rather than fixed.
  All four sit on the abandoned testing-infrastructure spec's
  tools/test-all.sh harness (bun-based, compose service dependencies,
  coverage gates vs test-config.yaml) and none has EVER concluded green in
  repo history. Their real coverage is already provided by the canonical
  surface: CI (fmt/clippy/check/cargo test + frontend typecheck/vitest/
  build/grep-gates), Live Integration Tier, BDD Chat Scenario Suite,
  Security Audit, and Eval Nightly. Sinking this cycle into a parallel
  harness that duplicates that coverage would be motion, not progress —
  and a customer-visible workflow that has never passed is worse than its
  absence. tools/test-all.sh itself remains for local use. Reverting is one
  commit if the operator disagrees.
- D5: template-cleanup.yml deleted outright (repo-template artifact).
- D6: live-integration.yml — root cause found: the workflow file has been
  INVALID YAML since it was committed (unquoted `cargo test --test
  integration live::` — the trailing colon parses as a mapping key), so
  every run failed at file-parse time before any step executed. Fixed by
  quoting the command.

## Risks / Trade-offs
- [Two audit ignore lists can still drift] → pointer comments both ways;
  full dedup into a shared file is a follow-up if drift recurs.
- [comprehensive-tests is a large workflow; more failures may hide behind the
  first three] → fix iteratively with real dispatches until green, disclose
  anything newly surfaced.

## Migration Plan
Workflow-only; rollback = revert.

## Open Questions
(none)
