# Focused verification evidence — `harden-jwt-defaults`

Date: 2026-08-18
Profile: `server-full`, except the separately named `uar-jwt-proxy` commands.

## Required fallback and anonymous control

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib built_in_fallback_secret -- --nocapture
```

Observed exit 0:

```text
running 2 tests
test config::tests::anonymous_mode_explicitly_allows_the_built_in_fallback_secret ... ok
test config::tests::required_jwt_rejects_the_built_in_fallback_secret ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 599 filtered out
```

## HS256 registered claims

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::security::middleware::tests::shared_secret_issuer_and_audience_are_required_when_configured -- --nocapture
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::security::middleware::tests::not_before_is_enforced_when_enabled_and_optional_when_disabled -- --nocapture
```

Observed exit 0 for each:

```text
test uar::security::middleware::tests::shared_secret_issuer_and_audience_are_required_when_configured ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 600 filtered out

test uar::security::middleware::tests::not_before_is_enforced_when_enabled_and_optional_when_disabled ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 600 filtered out
```

## UAR-issued token continuity

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::security::api_keys::tests::exchanged_jwt_contains_configured_issuer_and_audience -- --nocapture
cargo test --locked -p uar-jwt-proxy app_state_mints_hs256_token -- --nocapture
```

Observed exit 0 for each:

```text
test uar::security::api_keys::tests::exchanged_jwt_contains_configured_issuer_and_audience ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 600 filtered out

test tests::app_state_mints_hs256_token ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out
```

## Focused compatibility suites

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::security:: -- --nocapture
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib config::tests:: -- --nocapture
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test config_integration -- --test-threads=1 --nocapture
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib config_manager::tests -- --nocapture
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --bin uar-sidecar -- --nocapture
```

Observed result tails, all exit 0:

```text
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 564 filtered out
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 581 filtered out
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 598 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Tier 0, lint, and structural checks

```bash
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full,vault
cargo check --locked -p uar-jwt-proxy
cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
cargo fmt --all -- --check
openspec validate harden-jwt-defaults --strict --no-interactive
git diff --check -- src/config.rs src/config_manager.rs src/server.rs src/uar/api/a2a/grpc.rs src/uar/api/a2a/handler.rs src/uar/security/api_keys.rs src/uar/security/middleware.rs src/uar/security/verifier/mod.rs tests/settings_persistence.rs tests/test_a2a_grpc.rs tools/uar-jwt-proxy/src/main.rs docs/configuration.md docs/dev-tools.md example.config.yaml website/docs/configuration.md website/docs/dev-tools/intro.md openspec/changes/harden-jwt-defaults .refiner/artifacts/harden-jwt-defaults
```

Observed:

```text
UAR check: exit 0; Finished `dev` profile; 3 recorded pre-existing warnings.
Vault-enabled UAR check: exit 0; Finished `dev` profile; 3 recorded pre-existing warnings.
Proxy check: exit 0; Finished `dev` profile.
Clippy: exit 0; `universal-agent-runtime` (lib) generated 572 warnings.
Formatting: exit 0; no output.
OpenSpec: Change 'harden-jwt-defaults' is valid.
Scoped diff check: exit 0; no output.
```

The Clippy result is not represented as warning-free. Full phase Tier 2 was not
run at this individual-change boundary.
