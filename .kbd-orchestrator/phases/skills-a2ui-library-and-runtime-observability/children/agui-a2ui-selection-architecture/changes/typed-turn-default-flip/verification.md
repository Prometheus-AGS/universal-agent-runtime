<!-- mirror of openspec/changes/typed-turn-default-flip/tasks.md -->
# Tasks — typed-turn-default-flip

scope: src/config.rs (harness mode default), settings schema and tests, docs release notes, .prometheus/decisions.md

## 0. Evidence gate

- [ ] 0.1 Attach `parity-report.json` from typed-turn-assembly showing zero unexpected differences across the corpus; record corpus size
- [ ] 0.2 Run the live smoke set in `shadow` mode; record command, output, and zero unexpected differences

## 1. Failing test first

- [ ] 1.1 Settings-default test asserts `HarnessConfig::default().mode == Typed` and that `legacy` still deserializes

## 2. Flip

- [ ] 2.1 Change the default to `typed`; add the deprecation note for `legacy` to the settings schema
- [ ] 2.2 Release notes entry naming the rollback setting

## 3. Verification

- [ ] 3.1 Tier 2: fmt check and full `cargo test --locked --no-default-features --features server-full` with the new default
- [ ] 3.2 Decision log entry with the evidence from 0.1 and 0.2
- [ ] 3.3 `openspec validate typed-turn-default-flip --strict`
