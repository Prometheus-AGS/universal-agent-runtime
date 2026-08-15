# B4 focused positive verification

Date: 2026-08-15
Profile: `server-full` only. These results transfer to no other profile.

## Scoped service matrix

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::runtime::skills::service::tests -- --test-threads=1
```

Observed output (exit 0):

```text
running 7 tests
test uar::runtime::skills::service::tests::agent_binding_set_before_load_filters_future_skills ... ok
test uar::runtime::skills::service::tests::builtin_delete_is_refused_and_user_delete_succeeds ... ok
test uar::runtime::skills::service::tests::builtin_scoped_state_survives_restart_reregistration ... ok
test uar::runtime::skills::service::tests::scoped_change_affects_next_match_without_mutating_existing_binding ... ok
test uar::runtime::skills::service::tests::scoped_state_resolves_conversation_then_agent_then_global_both_directions ... ok
test uar::runtime::skills::service::tests::update_skill_modifies_selected_fields ... ok
test uar::runtime::skills::service::tests::update_skill_returns_none_for_missing_skill ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 573 filtered out; finished in 0.17s
```

## Cold restart, durable delete, and builtin refusal

The parent test launches separate `seed`, `reopen-delete`, and `verify-deleted`
processes against one SurrealKV directory and one filesystem skill directory.

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_scoped_governance scoped_state_and_user_deletion_survive_cold_restart -- --exact --test-threads=1
```

Observed output after exact source restoration (exit 0):

```text
running 1 test
test scoped_state_and_user_deletion_survive_cold_restart ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.31s
```

## Real-run live effect and binding stability

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_scoped_governance conversation_enable_widens_global_disable_and_in_flight_binding_is_stable -- --exact --test-threads=1
```

Observed output (exit 0):

```text
running 1 test
test conversation_enable_widens_global_disable_and_in_flight_binding_is_stable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.29s
```

## Filesystem scoped-config serialization

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::runtime::skills::storage::filesystem::tests::scoped_config_round_trips_through_skill_markdown -- --exact --test-threads=1
```

Observed output (exit 0):

```text
running 1 test
test uar::runtime::skills::storage::filesystem::tests::scoped_config_round_trips_through_skill_markdown ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 578 filtered out; finished in 0.00s
```

## API origin serialization

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::api::skills::tests::skill_response_exposes_lowercase_origin -- --exact --test-threads=1
```

Observed output (exit 0):

```text
running 1 test
test uar::api::skills::tests::skill_response_exposes_lowercase_origin ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 578 filtered out; finished in 0.00s
```

## Existing agent-binding API compatibility

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skills_api_integration_test set_agent_skills -- --test-threads=1
```

Observed output (exit 0):

```text
running 2 tests
test set_agent_skills_overwrites_previous_bindings ... ok
test set_agent_skills_replaces_bindings ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s
```

The seven-test service matrix above also observed
`agent_binding_set_before_load_filters_future_skills`: a pending legacy
allowlist selected a skill loaded later, excluded another later-loaded skill,
and still yielded to a conversation override.

## Tier 0

```bash
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
```

Observed output (exit 0):

```text
warning: `universal-agent-runtime` (lib) generated 3 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.98s
```

```bash
cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
```

Observed output (exit 0):

```text
warning: `universal-agent-runtime` (lib) generated 573 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.76s
```

The short-format replay also exited 0. It emitted existing diagnostics in files
that B4 touches, but none points at a line added by B4.

## Strict OpenSpec validation

```bash
openspec validate skill-scoped-governance --strict
```

Observed output (exit 0):

```text
Change 'skill-scoped-governance' is valid
```
