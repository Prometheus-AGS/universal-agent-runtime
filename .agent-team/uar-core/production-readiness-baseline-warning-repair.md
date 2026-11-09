# Baseline warning repair handoff

Phase: `production-readiness-476-baseline`, Execute.
Verification: **CLOSED_DURING_OBSERVED_BASELINE_REPAIR_COHORT**.
Source checkout: `/Users/gqadonis/.claude/worktrees/production-readiness-476`.

## Owned changes

- `tests/support/sidecar_process.rs`: removed unused `std::io::Write as _` and `std::time::Instant` imports. The remaining synchronous writer call is fully qualified as `std::io::Write::write_all`; asynchronous writes retain `tokio::io::AsyncWriteExt`. No platform-specific import is needed.
- `tests/bauar_resource_grants.rs`: added a reasoned `#[expect(dead_code)]` only on its shared `stub_llm` module inclusion. This target does not construct `DelayedContent` or call `with_prompt_prefix`.
- `tests/integration.rs`: added a reasoned `#[expect(dead_code)]` only on its `live` module inclusion. Its declared case modules omit the shared `run_scoped`, `DelayedContent`, and `with_prompt_prefix` fixture consumers.
- This handoff records the bounded repair and its limits.

The named expectations trace to the compiler warnings retained in `.kbd-orchestrator/phases/production-readiness-476-baseline/evidence/full-inventory-attempt-001.txt`. No new runtime guard, assertion change, blanket `allow`, lowered lint, or unrelated cleanup was added. Existing shared-helper source and other writers' files were preserved. Nothing outside the assigned scope was added.

## Source inspection and command classification

Commands were source inspection only: `pwd`, `cat` for waypoint/rules/skills/plan/source, `sed` for relevant source and gotchas, and `rg`/`rg --files` for actual module layout, imports, call sites, and retained diagnostics. `apply_patch` performed authorized source/documentation writes. No verification command was issued; no Cargo, rustc, formatter, lint, test, build, script, external gate, or `git diff --check` ran.

The waypoint reports `IN_PROGRESS`, Execute, and a closed repair-cohort gate. The complete plan names this warning repair as part of the same observed fixture-compatibility repair cohort. This handoff does not change canonical phase state or open the gate.

## Remaining decisions and unverified limits

No unresolved implementation decision remains in this ownership scope. Compilation, warning elimination, absence of unfulfilled lint expectations, formatting, and test behavior remain unverified because the gate is closed. Scope-limited `expect` attributes intentionally preserve warnings in other including targets. A module-level expectation is fulfilled by any matching warning in that target's included scope; it does not independently track every named helper.

Lead must collect all cohort handoffs, renew the whole-phase inventory with exact source identity, and follow the phase's authorization before review or the serial gate. No phase acceptance is claimed here.
