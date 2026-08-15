# A1 fail-closed negative controls

Date: 2026-08-14
Profile: `server-full` only.

The complete positive middleware diff was retained before either mutation as
`middleware-pre-inversion.diff`. Its SHA-256 is
`95bed2720b8ef8638c79e7545a5fa1148355b9a5a80d1f2e6fcc1d87447f146b`.
After each control was restored, the following complete-diff comparison exited
0 with no output:

```bash
git diff -- src/uar/security/middleware.rs |
  diff -u openspec/changes/gap-02-jwks-token-verifier/evidence/middleware-pre-inversion.diff -
```

## Verification-error branch

Only the required-token verification-error branch was inverted: required
requests returned the anonymous context while the disabled path remained
anonymous. The focused middleware group exited 101:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  uar::security::middleware::tests:: -- --test-threads=1
```

Observed output:

```text
running 10 tests
test uar::security::middleware::tests::jwks_token_authenticates_through_middleware_resolution ... ok
test uar::security::middleware::tests::jwks_unknown_kid_maps_to_unauthorized ... FAILED
test uar::security::middleware::tests::jwks_wrong_audience_maps_to_unauthorized ... FAILED
test uar::security::middleware::tests::jwks_wrong_issuer_maps_to_unauthorized ... FAILED
test uar::security::middleware::tests::test_resolve_user_context_anonymous_when_jwt_disabled_and_invalid_header ... ok
test uar::security::middleware::tests::test_resolve_user_context_anonymous_when_jwt_disabled_and_no_header ... ok
test uar::security::middleware::tests::test_resolve_user_context_rejects_token_signed_with_wrong_secret ... FAILED
test uar::security::middleware::tests::test_resolve_user_context_unauthorized_when_jwt_required_and_no_header ... ok
test uar::security::middleware::tests::test_resolve_user_context_valid_token ... ok
test uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed ... FAILED

failures:

---- uar::security::middleware::tests::jwks_unknown_kid_maps_to_unauthorized stdout ----

thread 'uar::security::middleware::tests::jwks_unknown_kid_maps_to_unauthorized' (1238322) panicked at src/uar/security/middleware.rs:327:14:
unknown kid must return 401: UserContext { user_id: "anonymous", claims: UserClaims { sub: "anonymous", name: Some("Anonymous"), roles: Some(["anonymous"]), exp: 18446744073709551615 } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- uar::security::middleware::tests::jwks_wrong_audience_maps_to_unauthorized stdout ----

thread 'uar::security::middleware::tests::jwks_wrong_audience_maps_to_unauthorized' (1238323) panicked at src/uar/security/middleware.rs:302:18:
wrong audience must return 401: UserContext { user_id: "anonymous", claims: UserClaims { sub: "anonymous", name: Some("Anonymous"), roles: Some(["anonymous"]), exp: 18446744073709551615 } }

---- uar::security::middleware::tests::jwks_wrong_issuer_maps_to_unauthorized stdout ----

thread 'uar::security::middleware::tests::jwks_wrong_issuer_maps_to_unauthorized' (1238324) panicked at src/uar/security/middleware.rs:315:18:
wrong issuer must return 401: UserContext { user_id: "anonymous", claims: UserClaims { sub: "anonymous", name: Some("Anonymous"), roles: Some(["anonymous"]), exp: 18446744073709551615 } }

---- uar::security::middleware::tests::test_resolve_user_context_rejects_token_signed_with_wrong_secret stdout ----

thread 'uar::security::middleware::tests::test_resolve_user_context_rejects_token_signed_with_wrong_secret' (1238327) panicked at src/uar/security/middleware.rs:277:14:
expected invalid signature to be rejected: UserContext { user_id: "anonymous", claims: UserClaims { sub: "anonymous", name: Some("Anonymous"), roles: Some(["anonymous"]), exp: 18446744073709551615 } }

---- uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed stdout ----

thread 'uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed' (1238330) panicked at src/uar/security/middleware.rs:355:14:
unreachable JWKS must fail closed: UserContext { user_id: "anonymous", claims: UserClaims { sub: "anonymous", name: Some("Anonymous"), roles: Some(["anonymous"]), exp: 18446744073709551615 } }

failures:
    uar::security::middleware::tests::jwks_unknown_kid_maps_to_unauthorized
    uar::security::middleware::tests::jwks_wrong_audience_maps_to_unauthorized
    uar::security::middleware::tests::jwks_wrong_issuer_maps_to_unauthorized
    uar::security::middleware::tests::test_resolve_user_context_rejects_token_signed_with_wrong_secret
    uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed

test result: FAILED. 5 passed; 5 failed; 0 ignored; 0 measured; 556 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

The source was restored with `apply_patch`, the complete-diff comparison above
exited 0, and the same focused command exited 0 with:

```text
running 10 tests
test uar::security::middleware::tests::jwks_token_authenticates_through_middleware_resolution ... ok
test uar::security::middleware::tests::jwks_unknown_kid_maps_to_unauthorized ... ok
test uar::security::middleware::tests::jwks_wrong_audience_maps_to_unauthorized ... ok
test uar::security::middleware::tests::jwks_wrong_issuer_maps_to_unauthorized ... ok
test uar::security::middleware::tests::test_resolve_user_context_anonymous_when_jwt_disabled_and_invalid_header ... ok
test uar::security::middleware::tests::test_resolve_user_context_anonymous_when_jwt_disabled_and_no_header ... ok
test uar::security::middleware::tests::test_resolve_user_context_rejects_token_signed_with_wrong_secret ... ok
test uar::security::middleware::tests::test_resolve_user_context_unauthorized_when_jwt_required_and_no_header ... ok
test uar::security::middleware::tests::test_resolve_user_context_valid_token ... ok
test uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 556 filtered out; finished in 0.04s
```

## Missing-token branch

Only the no-header required/optional results were inverted, then this command
was run:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  uar::security::middleware::tests::test_resolve_user_context_unauthorized_when_jwt_required_and_no_header \
  -- --exact --test-threads=1
```

Observed output:

```text
test uar::security::middleware::tests::test_resolve_user_context_unauthorized_when_jwt_required_and_no_header ... FAILED
expected unauthorized: UserContext { user_id: "anonymous", claims: UserClaims { sub: "anonymous", name: Some("Anonymous"), roles: Some(["anonymous"]), exp: 18446744073709551615 } }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 565 filtered out
error: test failed, to rerun pass `-p universal-agent-runtime --lib`
```

The source was restored with `apply_patch`, the complete-diff comparison exited
0, and the same exact command exited 0:

```text
test uar::security::middleware::tests::test_resolve_user_context_unauthorized_when_jwt_required_and_no_header ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 565 filtered out
```

## Final positive slice and error log

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  uar::security:: -- --test-threads=1
```

Observed output:

```text
running 33 tests
test uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed ... ok
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 533 filtered out
```

The unreachable-JWKS test installs an error-only tracing subscriber, captures
the actual event, and asserts that the captured output contains both `ERROR`
and `JWKS refresh failed`. That assertion executed in the passing test above.
