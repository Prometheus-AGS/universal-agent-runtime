# Positive verification evidence

Date: 2026-08-15
Profile: `server-full` only. These results transfer to no other profile.

## Skills unit slice

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib uar::runtime::skills -- --test-threads=1
```

Actual output:

```text
running 46 tests
test uar::runtime::skills::registry::tests::tombstoned_skill_is_hidden_but_retrievable_for_restore ... ok
test uar::runtime::skills::registry::tests::vector_candidates_include_tombstones_before_visibility_filtering ... ok
test uar::runtime::skills::service::tests::api_skill_survives_empty_configuration_by_provider_id ... ok
test uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin ... ok
test uar::runtime::skills::service::tests::reconciliation_adds_changes_tombstones_and_restores_scoped_config ... ok
test uar::runtime::skills::service::tests::reconciliation_survives_cold_process_restarts ... ok
test uar::runtime::skills::service::tests::refresh_hides_tombstones_but_keeps_them_available_for_restore ... ok
test uar::runtime::skills::storage::filesystem::tests::cold_reload_preserves_api_and_config_provenance ... ok
test uar::runtime::skills::storage::filesystem::tests::configuration_source_wins_over_a_stale_dynamic_copy ... ok
test uar::runtime::skills::storage::filesystem::tests::dynamic_storage_rejects_non_api_skills ... ok
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 545 filtered out; finished in 0.90s
```

The cold-process test launched four separate child processes (`seed`, `change`,
`remove`, and `restore`) against one SurrealKV directory. Each child reopened
the database rather than reusing a live provider handle.

## Pre-review restored service candidate

Command:

```bash
shasum -a 256 src/uar/runtime/skills/service.rs
git diff -- src/uar/runtime/skills/service.rs | shasum -a 256
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib uar::runtime::skills::service::tests \
  -- --test-threads=1
```

Actual output:

```text
96920a864e47f267126849b245d77c1cfd8ff52b2fc99d5cc790de5b05a74472  src/uar/runtime/skills/service.rs
21e890e94eb60b7a0b731e9b3e86f6ee745a7a65d26e62a68b7a4a7be2b0eb6d  -
running 11 tests
test uar::runtime::skills::service::tests::api_skill_survives_empty_configuration_by_provider_id ... ok
test uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin ... ok
test uar::runtime::skills::service::tests::reconciliation_adds_changes_tombstones_and_restores_scoped_config ... ok
test uar::runtime::skills::service::tests::reconciliation_survives_cold_process_restarts ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 576 filtered out; finished in 0.82s
```

The independent review then found the visibility and filesystem-boundary defects
covered by the four later controls. After their correction, exact restoration
was observed as:

```text
9956eca135efc92ea4629269a14e04d7f5c83135c53a684f890635bd563c6a14  src/uar/runtime/skills/registry.rs
cd9324777bbfe070e5f48746e835bf91adc885442bb6d6de158c622bfcc82303  src/uar/runtime/skills/service.rs
dbb88829501b32924988b81a933153695927bfc3ba57953f546c472504f79585  src/uar/runtime/skills/storage/filesystem.rs
ef22852b7ed647a02669f591fa7b8c03d45381209c96ff911ee3aaa196c3ea64  -
```

The complete 46-test skills slice shown above passed on those hashes.

After the critic required observed error-level fail-safe logging, the test-only
subscriber changed only `service.rs`. The final candidate hashes are:

```text
9956eca135efc92ea4629269a14e04d7f5c83135c53a684f890635bd563c6a14  src/uar/runtime/skills/registry.rs
974320697dcc844f6ef44c40e18ad7679dab45bc57cf6b33a857d26acdfdca0a  src/uar/runtime/skills/service.rs
dbb88829501b32924988b81a933153695927bfc3ba57953f546c472504f79585  src/uar/runtime/skills/storage/filesystem.rs
f6f9e87294896631b5b3b5f27dbd07f08c670fe39400698c0685397e885ac72f  -
```

The complete skills slice again observed `46 passed; 0 failed` on these final
hashes.

## Tombstone domain invariant

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::domain::skills::tests::tombstone_overrides_scope_without_destroying_configuration \
  -- --exact --test-threads=1
```

Actual output:

```text
running 1 test
test uar::domain::skills::tests::tombstone_overrides_scope_without_destroying_configuration ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 586 filtered out; finished in 0.00s
```

## Attributable tombstone log

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::service::tests::reconciliation_adds_changes_tombstones_and_restores_scoped_config \
  -- --exact --nocapture --test-threads=1
```

Actual output:

```text
INFO universal_agent_runtime::uar::runtime::skills::service: tombstoned configuration-managed skill skill_id=config-removed reason="absent_from_configuration"
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 586 filtered out; finished in 0.26s
```

## Empty-source error log

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib \
  uar::runtime::skills::service::tests::empty_source_fail_safe_preserves_every_skill_origin \
  -- --exact --nocapture --test-threads=1
```

Actual output (exit 0):

```text
ERROR universal_agent_runtime::uar::runtime::skills::service: configuration skill source is empty; refusing to tombstone stored skills stored_config_skills=1
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.15s
```

## Tier 0

Commands:

```bash
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
cargo clippy --locked -p universal-agent-runtime --no-default-features \
  --features server-full --lib --no-deps --message-format short
```

Actual output:

```text
warning: `universal-agent-runtime` (lib) generated 3 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.86s

warning: `universal-agent-runtime` (lib) generated 573 warnings (2 duplicates)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.09s
```

Both commands exited 0. The 573-warning Clippy baseline was restored after the
single B5-introduced `collapsible_if` warning was fixed.

## OpenSpec and formatting

Commands:

```bash
openspec validate skill-config-reconciliation --strict
cargo fmt --all -- --check
git diff --check
```

Actual output:

```text
Change 'skill-config-reconciliation' is valid
```

All three commands exited 0; the formatting and diff checks emitted no output.
