# provisioning-tests-and-docs

## Why

Closes the redesigned `integration-tests-and-docs` (#14) for this
phase's reduced scope. The Spec Gap Summary from `assessment.md`
flagged that no canonical doc explained the new provisioning system's
contract; `provisioning-strategy-core`'s own proposal disclosed a real
test gap (native-package-manager/git-install/prebuilt-binary strategies
aren't exercised end-to-end, since doing so in an automated unit test
would modify the CI host).

## What changed

- New `docs/PROVISIONING.md`: the 4-strategy precedence, why it's
  separate from `process_supervisor.rs`, `ProvisionOptions::allow_install`'s
  gating rationale, how to add a new `ToolSpec`, and explicit non-goals
  (no database engines; not wired to skill compilation, since no such
  code path exists; `surreal-memory`/`liter-llm` aren't provisioned
  targets).
- `docs/ARCHITECTURE.md`: new **D-E** entry in the Architectural
  Decisions section, cross-referencing `docs/PROVISIONING.md` and
  stating plainly that `process_supervisor.rs` and `provisioning.rs`
  solve different problems and are not merged.
- `src/uar/orchestrator/provisioning.rs`: one new `#[ignore]`d test,
  `git_install_clones_and_builds_from_a_local_fixture_repo`, closing
  Round 1's disclosed gap for the `GitInstall` strategy specifically —
  it builds a real, throwaway local git repo (via `git init` +
  `git commit`, no network) with a trivial `build.sh` that produces an
  output binary, then runs the actual `git_install()` function against
  it end to end (clone → run build command → verify the resulting
  binary path exists). This is `#[ignore]`d because it still shells out
  to `git` and a build command — more than a pure unit test should
  assume is always safe/fast in every CI environment — but is safe to
  run manually since it never touches the network or the real host's
  package managers.

## Verification

- `cargo test --lib`: 385/385 green, 1 ignored (the new fixture test).
- `cargo test --lib -- --ignored git_install`: 1/1 green, confirmed
  run explicitly (not just compiled) — the `GitInstall` strategy is
  now exercised end-to-end, not just at the error-path level.
- `cargo clippy --lib`: 502 warnings, unchanged from baseline.
- `cargo clippy --lib --tests`: caught and fixed one real new lint
  (`doc_markdown` — missing backticks around `GitInstall` in a doc
  comment) introduced by the new test; zero `provisioning.rs` warnings
  after the fix, in both `--lib` and `--lib --tests` scope.
- Remaining disclosed gap (unchanged from `provisioning-strategy-core`):
  native-package-manager installation and prebuilt-binary download+
  extract are still not exercised end-to-end by an automated test —
  doing so would modify the CI host's real package state or require
  live network access. `docs/PROVISIONING.md` states this plainly as
  the "Non-goals"/test-coverage section rather than leaving it
  implicit.

This closes out Round 3, the final round of `prometheus-package-integration`'s redesigned `#4` scope.
