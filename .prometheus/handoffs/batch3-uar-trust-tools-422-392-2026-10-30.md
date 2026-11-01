# Handoff: uar-trust-tools, issues #422 (JWT validator contract) and #392 (local identity)

Date: 2026-10-30. Branch: fix/batch3-identity. Role: uar-trust-tools.

## Verification status

NOTHING WAS COMPILED OR RUN. No cargo (check/build/test/clippy/fmt), test,
linter, `git diff --check` or gate script was executed (verification lock
CLOSED). Everything below is source inspection. The one non-source command was
`openssl` generating throwaway EC fixtures into the scratchpad. They were later
removed from the change (see "Tests"), so no generated key material is in the
repo.

## Changed paths (post-edit line ranges)

1. `src/uar/security/verifier/mod.rs`
   - L8: import `Header`.
   - L86-131: new `VerificationError::UnsupportedTokenType`;
     `VerificationError::is_key_source_unavailable()` (true for
     `Jwks(Unavailable|Stale|Timeout|InvalidKeySet)`, false for
     `Jwks(UnknownKeyId)`); `ACCESS_TOKEN_TYPES`; `ensure_access_token_type`
     (J08).
   - L200-202: shared-secret verifier checks `typ` (via `decode_header`) before decode.
   - L310-313: JWKS verifier checks `typ` before the algorithm allowlist and key lookup.
2. `src/uar/security/middleware.rs`
   - L9, L14: imports `ConnectInfo`, `SocketAddr`. L18-21: `pub const LOCAL_LOOPBACK_SUBJECT = "local-user"`.
   - L124-157: `resolve_authenticated_context` maps every verification failure
     through the new `verification_failure_status`. No anonymous fallback for a
     presented bearer (decision A).
   - L161-226: `trusted_local_loopback_context`, `is_loopback_authority`,
     `names_loopback_origin` (decision B).
   - L366-369: `auth_middleware` uses the loopback owner in the
     no-Authorization / no-x-api-key branch, before the anonymous fallback.
   - Tests: L426/L428 imports; L554-563 renamed and inverted test; L734-735
     expectation 401→503; L747-1029 new tests.
3. `src/uar/security/claims.rs`
   - L45-48: `CredentialKind::LocalLoopback`. L97-99: `VerifiedIdentity::is_local_loopback()`.
   - L187-208: `UserContext::trusted_local_loopback(subject)` (cfg `server`).
   - L250, L255-264: `CredentialPrincipal::local_host` helper, reused by the existing sidecar branch (behavior unchanged).
4. `src/uar/security/api_keys/policy.rs` L100-103: `KeyPolicy::identity` refuses the loopback owner (no API-key create/list/revoke).
5. `src/server.rs` L3426-3431 and L3443-3446: only the two `axum::serve` calls now use `into_make_service_with_connect_info::<std::net::SocketAddr>()`. Both listeners are `tokio::net::TcpListener`, which axum 0.8.9 implements `Connected` for (`axum-0.8.9/src/extract/connect_info.rs:90`).
6. Tests: `src/uar/security/verifier/tests.rs` (imports L1-12, new tests L260-612), `src/uar/security/verifier/test_support.rs` (L197-229 helpers), `src/uar/security/api_keys/tests/management.rs` (L354-375).
7. New doc: `docs/security/jwt-validation.md`.

## Interface effects

- HTTP, `jwt_required=false`: a presented bearer that does not verify is now 401
  (previously anonymous). A configured JWKS that is unavailable, failing, timed
  out, invalid or past hard age is now 503 for every `jwt_required` value
  (previously 401, or anonymous when optional). An unknown `kid` against a
  fresh key set stays 401. That is a deliberate reading of decision A's
  "`Jwks(_)` → 503": it is a credential defect, not unavailability, and keeps
  `jwks_unknown_kid_maps_to_unauthorized`. ProviderConflict→500 and
  IdentityPolicy→503 are unchanged.
- `authenticate_bearer_context` inherits the same mapping.
- gRPC A2A (`src/uar/api/a2a/grpc.rs:64-70`, not owned) is unchanged. It still
  maps JWKS unavailability to `UNAUTHENTICATED`. Documented, not changed.
- J08: an explicit `typ` must be `JWT`, `application/jwt`, `at+jwt` or
  `application/at+jwt` (case-insensitive). `application/jwt` is accepted beyond
  the literal decision C list. It is the RFC 7515 §4.1.9 full form of `JWT`,
  parallel to `application/at+jwt`. Remove it from `ACCESS_TOKEN_TYPES` and the
  `j08_access_token_typ_values_and_absent_typ_are_accepted` list if the
  operator wants the literal list.
- Decision B: a credential-less request becomes the `local-host` owner
  (`issuer_kind local-host`, `urn:uar:local:host`, tenant none, subject
  `local-user`) when all of these hold: the TCP peer is loopback (`ConnectInfo`
  only), `jwt_required=false`, `trusted_local`, there is no `Authorization` and
  no `x-api-key`, AND the request names a loopback `Host` (`localhost`,
  loopback IP, or `[::1]`) with no `Origin` or only a loopback `Origin`.
  - The Host/Origin condition is hardening I added at a real trust boundary;
    it was not in decision B. Named failure: a DNS-rebinding page arrives
    under its own host name, and a cross-origin page carries its own
    `Origin`. Both reach the loopback socket, and without the check either
    would read the local owner's history, memory and KB. The threat is
    already documented in `src/uar/security/sidecar_guard.rs:1-10` and
    enforced there (`:225-242`, Origin refusal in `enforce`). Headers only
    refuse here. They never establish loopback.
- The loopback owner carries no `HostAuthority` (`trusted_host_id()` is None)
  and no roles. A `HostAuthority` would have granted host administration
  (`src/uar/api/memory_admin.rs:108ff`, `src/uar/api/supervisor.rs:40,52`,
  `src/uar/api/accounting.rs:45`, `src/uar/admin/memory.rs:57ff`,
  credential migration `src/uar/security/credentials/migration.rs:108`, remote
  MCP grants `src/uar/runtime/turn/host/mcp.rs:353`). Those stay denied.
  Instead it carries a `VerifiedIdentity` of kind `LocalLoopback` with a
  captured `local-host` principal. This is the same mechanism as
  `VerifiedIdentity::for_scoped_delegation` (`claims.rs:73-81`).

## Owner-key trace for B (source evidence)

Every path derives the key from `CredentialPrincipal::from_verified_context`
(`claims.rs:226`), which returns the captured `local-host` principal
(`claims.rs:231-233`; `verified_identity()` matches because subject, tenant and
roles agree, `claims.rs:104-109`). `storage_key()` (`claims.rs:270ff`) is
therefore identical across:

- History: `UserContext::session_owner_key` (`claims.rs:169-176`, non-anonymous
  branch). HTTP handlers: `src/server.rs:4219,4576,5034,6686`. AG-UI:
  `src/uar/api/agui.rs:95`. Runs: `ActorOwner::from_verified_context`
  (`src/uar/runtime/actor/messages.rs:44-60`) keeps `credential_principal`,
  `ActorOwner::session_owner_key` (`messages.rs:126-129`) is consumed at
  `src/uar/runtime/manager.rs:4681-4684`.
- Memory: `src/uar/memory/scopes.rs:19-21` and `MemoryService::for_context`
  (`src/uar/memory/service.rs:1457-1460`).
- Knowledge base: `src/uar/api/knowledge.rs:203,231,279` (`session_owner_key`)
  and `src/uar/api/discovery.rs:785`.
- Tools / credentials: `src/uar/tools/session_search.rs:87`; the run
  credential binding `src/uar/runtime/manager.rs:925-955` →
  `CredentialTenancyPolicy::resolve_source` / `owner_for_capture`
  (`src/uar/security/credentials/policy.rs:134-138`) →
  `CredentialOwnerBinding::bind` (`src/uar/security/credentials/owner.rs:14-22`).
- Same key as a sidecar launch-bound context with the same subject. The test
  `j10_loopback_without_credential_gets_one_local_host_owner_for_all_stores`
  asserts history == memory == tool/run == principal == sidecar key.

Conflicts and limits found in the trace (reported, not widened):

- Stored provider credentials for any `local-host`/`local-jwt` owner require
  `security.provider_credentials.installation_id` (`owner.rs:18`). When
  multi-tenant provider credentials are enabled (`ProviderService::from_env`
  returns Some, `src/server.rs:1074-1091`) and no installation id is set,
  loopback runs now fail with `credential_installation_id_required`, where
  anonymous runs used house credentials. This is the existing local-owner
  contract. It is an operator configuration step, not code.
- `/api/uar/credentials` for the loopback owner returns 503
  `credential_installation_id_required` without an installation id
  (`src/uar/api/credentials.rs:44-47`). Credential migration preview returns
  403 (was 401).
- Remote MCP grants still require a trusted host. The loopback owner is
  refused, as anonymous was.

## UNRESOLVED CONFLICT: live / BDD harness expectations (tests/**, not owned)

The live harness boots a real server with `jwt_required: false`, host
`127.0.0.1`, default `trusted_local` (`tests/integration/live/harness.rs:524-531`).
`tests/common/test_server.rs:24-31` is the same. reqwest sends
`Host: 127.0.0.1:<port>` and no `Origin`, so every credential-less harness
request is now the `local-host` owner under decision B. These assertions
encode the old anonymous refusal and are expected to flip:

- `tests/integration/live/baseline_cases.rs:645-656`: anonymous
  `GET /api/uar/credentials` expects 401. It will likely get 503
  (`credential_installation_id_required`) or 200.
- `tests/integration/live/capability_cases.rs:577-591` (`l3_c04_credentials_listing`):
  expects 401. Same flip.
- `tests/integration/live/capability_cases.rs:966-976` (`l3_c02_...`): a
  credential-less direct tool execute expects 401. It will be admitted as the
  local owner.
- `tests/integration/live/capability_cases.rs:1300-1310` (C-21): unauthenticated
  ACP expects 401. `src/uar/api/acp/routes.rs:99` refuses only
  `ANONYMOUS_SESSION_OWNER`, so the call will be admitted.
- `tests/integration/live/capability_cases.rs:2199-2237`: still holds (the
  owner bucket is `local-user` instead of anonymous; peer isolation holds),
  but its comments say anonymous.
- `tests/bdd.rs` scenarios that rely on anonymous loopback behavior were not
  enumerated.

The lead and operator must choose: update these assertions to the decision-B
contract, or narrow decision B (for example opt-in configuration). I did not
edit tests/** or add configuration.

The other #392 data effect: history that loopback callers stored under
`anonymous` before this change is not visible to the `local-host` owner. There
is no migration. #392 asks to "retain stored data compatibility". The data is
retained but not adopted. Needs an operator decision.

## J01-J10 matrix

| ID | Existing tests | New tests |
| --- | --- | --- |
| J01 | middleware `test_resolve_user_context_valid_token` (HS256), `jwks_token_authenticates_through_middleware_resolution` (RS256); verifier `accepts_two_cached_keys_and_refreshes_after_rotation`; `tests/jwks_es256_verification.rs` `es256_token_from_jwks_authenticates_and_owns_its_task`, `es384_token_from_jwks_authenticates`, `rs256_token_from_jwks_still_authenticates` | (none; valid path also re-asserted in `j10_invalid_bearers_are_unauthorized_never_anonymous_when_jwt_optional`) |
| J02 | `test_resolve_user_context_rejects_token_signed_with_wrong_secret`; authority `remote_registered_claims_and_identity_are_required`; `es256_token_signed_by_unpublished_key_is_unauthenticated` | `j02_tampered_payload_with_original_signature_is_rejected` |
| J03 | authority `remote_registered_claims_and_identity_are_required` (HS384); `es256_token_against_rsa_jwk_is_unauthenticated`, `rs256_token_against_ec_jwk_is_unauthenticated`, `hs256_token_is_unauthenticated_on_the_jwks_path`, `ec_jwk_declaring_a_contradicting_alg_is_not_trusted` | `j03_alg_none_is_rejected_on_both_verifiers`, `j03_disallowed_algorithms_are_rejected_on_both_verifiers` |
| J04 | `shared_secret_issuer_and_audience_are_required_when_configured`; authority remote expired case | `j04_missing_malformed_or_expired_exp_is_rejected`, `j04_missing_or_malformed_required_iss_and_aud_are_rejected` |
| J05 | `not_before_is_enforced_when_enabled_and_optional_when_disabled`; authority remote nbf/exp cases | `j05_exp_inside_sixty_second_leeway_is_accepted_and_outside_is_rejected`, `j05_nbf_absent_or_inside_leeway_is_accepted_and_outside_is_rejected` |
| J06 | `rejects_wrong_issuer_and_audience`, `jwks_wrong_audience_maps_to_unauthorized`, `jwks_wrong_issuer_maps_to_unauthorized`, `es256_token_with_wrong_audience_is_unauthenticated` | `j06_audience_array_membership_is_accepted_and_non_membership_rejected` |
| J07 | `unknown_kid_refreshes_once_then_rejects`, `jwks_unknown_kid_maps_to_unauthorized`, `failed_refresh_never_renews_age_and_hard_age_denies`, `production_verifier_bounds_construction_and_delayed_fetch`, `concurrent_unknown_keys_share_one_refresh_and_retry_interval`, `target_refresh_removes_absent_keys_and_replaces_same_kid_material`, `unreachable_jwks_with_no_cache_fails_closed` (EXPECTATION CHANGED 401→503) | `j07_token_jku_and_x5u_headers_never_redirect_key_lookup`, `j07_unavailable_configured_jwks_is_classified_unavailable`, `j07_unreachable_jwks_is_service_unavailable_when_jwt_optional` |
| J08 | (none) | `j08_malformed_compact_jwts_are_rejected`, `j08_explicit_non_access_typ_is_rejected_before_key_lookup`, `j08_access_token_typ_values_and_absent_typ_are_accepted` |
| J09 | authority `incomplete_remote_policy_never_falls_back_to_anonymous`; `src/config.rs` config tests | `j09_misconfigured_identity_policy_is_service_unavailable_regardless_of_jwt_required`, `j09_repeated_equivalent_credential_failures_map_consistently` |
| J10 | `test_resolve_user_context_anonymous_when_jwt_disabled_and_no_header`, `test_resolve_user_context_unauthorized_when_jwt_required_and_no_header`, `sidecar_bound::*` | `j10_invalid_bearer_is_unauthorized_when_jwt_optional` (REPLACES `test_resolve_user_context_anonymous_when_jwt_disabled_and_invalid_header`, inverted), `j10_invalid_bearers_are_unauthorized_never_anonymous_when_jwt_optional`, `j10_verification_paths_leak_no_token_or_key_material`, `j10_loopback_without_credential_gets_one_local_host_owner_for_all_stores`, `j10_non_loopback_credentialed_or_strict_requests_get_no_local_owner`, `j10_trusted_local_loopback_owner_cannot_create_or_list_api_keys` |

Test modules: verifier tests are `uar::security::verifier::tests`, middleware
tests are `uar::security::middleware::tests`, and the API-key test is
`uar::security::api_keys::tests::management`.

I removed duplicate ES256/ES384 and key-type-substitution tests I had first
written once I found `tests/jwks_es256_verification.rs`. The EC fixtures went
with them.

J10 limit: the unit tests exercise `resolve_authenticated_context` and
`trusted_local_loopback_context`, the two functions `auth_middleware` calls.
They do not run `auth_middleware` itself, because `AppState` has no test
constructor. The real ingress is exercised end to end only by the live
harness, whose expectations conflict (above).

#392 two-remote-tenant coverage: existing
`uar::security::authority::scenarios::verified_identity_requires_exact_workspace_mapping`.
Nothing new.

## Exact commands for the lead's gate (NOT RUN)

New and changed tests:

```
cargo test --locked --no-default-features --features server-full --lib uar::security::verifier::tests::j0
cargo test --locked --no-default-features --features server-full --lib uar::security::middleware::tests::j
cargo test --locked --no-default-features --features server-full --lib uar::security::middleware::tests::unreachable_jwks_with_no_cache_fails_closed
cargo test --locked --no-default-features --features server-full --lib uar::security::api_keys::tests::management::j10
```

Bounded regression inventory (#422):

```
cargo test --locked --no-default-features --features server-full --lib uar::security::verifier
cargo test --locked --no-default-features --features server-full --lib uar::security::middleware
cargo test --locked --no-default-features --features server-full --lib uar::security::authority
cargo test --locked --no-default-features --features server-full --lib uar::security::api_keys
cargo test --locked --no-default-features --features server-full --lib uar::security::jwt
cargo test --locked --no-default-features --features server-full --test jwks_es256_verification
```

`jwks_es256_verification` requires `a2a-transport`, which `server-full`
includes. Compile risk to check first:

- `IntoMakeServiceWithConnectInfo` in the `tokio::try_join!` of the two serve futures.
- The `ConnectInfo(peer)` closure pattern.
- `IpAddr::to_canonical`.
- The `Header { typ, ..Header::default() }` struct update.
- The new `CredentialKind::LocalLoopback` dead-code status in non-`server` builds. It is read via `==`, so no warning is expected.

## Not done / unverified

- Nothing compiled. Warnings and errors are unknown. rustfmt was not run. The
  touched files were already not rustfmt-clean before this change.
- The JWKS verifier accepts ES256/ES384 even when `jwt_algorithm: RS256` is
  configured (`validate_identity_policy` treats RS256 as "the JWKS verifier"). I
  documented this instead of changing it. Narrowing it would break ES deployments
  in `remote`.
- An `Authorization` header with a non-`Bearer ` scheme, or lowercase
  `bearer`, is still treated as "no bearer": anonymous when JWT is optional. It
  never becomes the loopback owner. Unchanged.
- `.prometheus/decisions.md` was not updated. The lead may record the
  `UnknownKeyId`→401, `application/jwt` and Host/Origin decisions there.

## Amendment: opt-in flag (operator decision, 2026-10-30)

The operator ruled that the loopback `local-host` owner ships as an opt-in
flag, default OFF. This resolves the "UNRESOLVED CONFLICT" section above.

Still NOTHING COMPILED OR RUN; the lock is still CLOSED.

### Changed lines

- `src/config.rs`
  - L761-770: new `SecurityConfig::local_loopback_owner: bool`
    (`#[serde(default)]`, so false). Its `///` doc says it is for
    single-user desktop/host installs only, never behind a same-host proxy or
    mesh sidecar (Istio 127.0.0.6, client-sent `Host: localhost`), and is
    ignored unless `trusted_local` and `jwt_required: false`.
  - L790: added to the redacted `Debug` impl.
- `src/uar/security/middleware.rs`
  - L160-185: `trusted_local_loopback_context` now requires
    `config.local_loopback_owner` (L181). With the flag off, a
    credential-less loopback request keeps the anonymous context unchanged.
    The Host/Origin refusal is kept for when the flag is on.
  - L766-773: new test helper `local_owner_config()`. All existing loopback
    tests now use it, including the strict and remote variants.
  - L979-1003: new test
    `j10_loopback_without_credential_stays_anonymous_when_local_loopback_owner_is_off`.
    It checks the explicit-false config and a serde config with the key
    absent. Loopback IPv4, IPv6 and loopback-Origin requests get no local
    owner; resolution yields `anonymous` with owner key `anonymous`.
- `docs/security/jwt-validation.md`: the flag is in the config table and the
  status table.
  - New "Local-host loopback identity (#392, opt-in)" text: default off and
    its preconditions.
  - Rewritten hazard paragraph: proxy, tunnel and mesh sidecar, Istio
    127.0.0.6, and `Host` is client data.
  - Data note: `anonymous` history is not migrated, and toggling the flag
    hides one or the other set of data.
  - Explicit rationale for unknown `kid` → 401 and for accepting
    `application/jwt`.
- `.prometheus/decisions.md`: appended the
  "2026-10-30 — JWT validator (#422) and local loopback owner (#392)" entry,
  covering the flag decision plus the three judgement calls.

### `SecurityConfig { .. }` struct literals touched

Each got `local_loopback_owner: false,`, inserted after `settings_admin_key`:

- `src/config.rs:2679`, `src/config.rs:2707`
- `src/uar/security/middleware.rs:277` (`resolve_user_context`), `:496`
  (`jwks_config`), `:518` (`shared_secret_config`)
- `src/uar/api/a2a/grpc.rs:536`, `src/uar/api/a2a/handler.rs:300`
- `tests/test_a2a_grpc.rs:117`, `tests/a2a_thread_service.rs:84`,
  `tests/settings_persistence.rs:163`, `tests/context_history_integrity.rs:2214`,
  `tests/jwks_es256_verification.rs:207`, `tests/stream_terminal_contract.rs:109`,
  `tests/support/truthful_tool_outcome_peer.rs:49`

Other `SecurityConfig` constructions build from serde JSON and get the
default: `tests/dot_memory_owner_contract.rs:21`,
`src/uar/runtime/turn/host/mcp_tests.rs:150`,
`src/uar/security/api_keys/tests/support.rs:3,73`, `authority.rs:66`. The
repository grep `SecurityConfig *{` outside `target/`, `vendor/` and worktrees
found no other literal.

### Live harness confirmation (read only)

Neither file sets the flag, so it is false there:

- `tests/integration/live/harness.rs:524-531` writes `security:` with
  `jwt_required`, `jwt_secret` and `settings_admin_key` only.
- `tests/common/test_server.rs:24-31` writes `jwt_required: false` and
  `jwt_secret` only.

Credential-less harness requests stay anonymous, so the live assertions in the
conflict list keep holding. `tests/integration/live/*` was not edited.

### Added gate command (NOT RUN)

```
cargo test --locked --no-default-features --features server-full --lib uar::security::middleware::tests::j10_loopback_without_credential_stays_anonymous_when_local_loopback_owner_is_off
```

It is also covered by the existing `uar::security::middleware::tests::j`
filter. Compile risk introduced by this amendment: struct literals compiled
under other features (`a2a-transport` tests). Every literal found was
updated.
