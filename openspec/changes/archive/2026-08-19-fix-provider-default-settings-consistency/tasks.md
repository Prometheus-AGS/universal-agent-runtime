## 1. Contract and Baseline

- [x] 1.1 Strict-validate the completed OpenSpec change and confirm the child KBD permitted surface still limits product edits to `src/uar/settings/manager.rs`, `src/uar/api/providers.rs`, and `tests/settings_persistence.rs`.

## 2. Settings Provider Consistency

- [x] 2.1 Add `mgr_initialize_accepts_local_memory_embedding_provider` and `mgr_initialize_rejects_unknown_memory_embedding_provider` to `tests/settings_persistence.rs`; observe the supported-value test fail before implementation and the unknown-value negative control reject the configuration, then run the required Tier 0 check.
- [x] 2.2 Add `local` to the closed `memory.embedding_provider` schema enum in `src/uar/settings/manager.rs`; rerun both exact settings tests and the Tier 0 check, recording their actual outputs.

## 3. Default Provider Publication Ordering

- [x] 3.1 Add focused handler tests for persistence failure preserving the live default, a missing provider preserving live and durable defaults, and successful selection surviving a fresh `SettingsManager`; observe the pre-fix ordering failure, then run the required Tier 0 check.
- [x] 3.2 Change the default-provider handler to validate the target, persist the selection when settings persistence is configured, and only then publish it to the registry; rerun all three exact handler tests and the Tier 0 check, recording their actual outputs.

## 4. Child Verification and Review

- [x] 4.1 Run the five focused tests together with `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full`, package-scoped Clippy with `--lib --no-deps`, `cargo fmt --all -- --check`, and scoped `git diff --check`.
- [x] 4.2 Run `openspec validate fix-provider-default-settings-consistency --strict --no-interactive` and write `verification.md` with the exact commands, outputs, per-requirement results, and paired negative controls.
- [x] 4.3 Produce and validate the artifact-refiner snapshot, then obtain history-free critic and judge approval of the implementation and evidence while the live OpenSpec change remains reviewable.

## 5. Sync and Parent Handoff

- [x] 5.1 Archive and sync the approved change with `openspec archive fix-provider-default-settings-consistency --yes`, then run strict validation of the canonical `provider-model-settings-certification` spec.
- [x] 5.2 Complete child reflection and `/kbd-child-exit complete`, replace `handoff-out.md` placeholders with exact deliverables and parent resume commands, and restore the parent exact next work to `/opsx:apply screen-by-screen-validation`.
- [x] 5.3 Append the decision and observed failure lesson to `.prometheus`, stage only the permitted child implementation/evidence/KBD/history artifacts while excluding `.refiner/registry.json` and unrelated user changes, and create the single child commit without pushing or opening a PR.
