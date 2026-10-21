# R8 native SDK acceptance diagnosis

The initial diagnosis below is retained as pre-repair evidence. The implementation
addendum at the end records the subsequent assigned repair and current status.

Phase: `uar-outstanding-issues-v1-1-1`; authoritative stage at inspection: Execute,
task `6.2`, waypoint revision 3006. Assignment: source-only diagnosis while the
lead's serial acceptance program remains active. Sole changed file: this handoff.
No production or fixture edits and no verification commands were performed.

## Observed result

Retained evidence: `evidence/acceptance/20261016-local-01/command-00014.json`
and its `.log` under this phase. The lead's recorded command was:

```text
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test credentials_api_integration_test phase_native_sdk_supplied_credentials_use_real_provider_and_durable_store -- --exact --test-threads=1
```

Recorded exit: **101**. Actual result:

```text
thread 'phase_native_sdk_supplied_credentials_use_real_provider_and_durable_store' (32223553) panicked at tests/support/credential_process.rs:480:67:
called `Result::unwrap()` on an `Err` value: 500
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 3.32s
```

Compilation finished before this test ran. The separate missing-Debug warning
on `CredentialApiState` is not the cause of this panic. This diagnostic agent
read the retained receipt; it did not execute or repeat the command.

## Concrete source trace

1. `tests/credentials_api_integration_test.rs:713–726` starts the provider and
   durable store, then calls `process::authenticated_context`. Embedded runtime
   construction begins later at line 743. Thus the failure precedes the tested
   SDK execution and both expected provider-contact outcomes.
2. `tests/support/credential_process.rs:474–480` creates `SecurityConfig` and
   evaluates `token(...)` as the argument to `authenticate_bearer_context`.
   `token` calls raw `jsonwebtoken::encode` at lines 264–269. There is no earlier
   UAR verifier initialization in this helper.
3. `src/uar/security/middleware.rs:116–118` enters
   `resolve_authenticated_context`; lines 87–100 invoke the verifier and map
   **only `VerificationError::ProviderConflict` to HTTP status 500**.
   Identity policy failure maps to 503; ordinary token rejection maps to 401.
   Consequently the observed 500 identifies the provider-conflict branch.
4. `src/uar/security/verifier/mod.rs:149–161` routes shared-secret verification
   into UAR's guarded `jwt::decode`; lines 88–93 preserve the provider-conflict
   classification.
5. `src/uar/security/jwt.rs:25–32` attempts to install the selected provider
   once, stores success or failure in `UAR_PROVIDER`, and fails closed if that
   first acquisition failed. `decode` calls this guard at line 53 before token
   decoding. A failed acquisition remains a conflict for the process lifetime.
6. Existing fixtures already account for the ordering:
   `tests/support/credential_process.rs:280–284` (`authenticated`) and
   lines 128–134 (`Node::boot`) invoke the public authentication adapter with
   `"not-a-token"` before signing synthetic input. Production server startup
   acquires the provider explicitly at `src/server.rs:472–476`.

The source-grounded reproducer is the retained exact single-test invocation:
start that isolated integration test without another fixture first initializing
UAR, reach the new helper, sign synthetic input, then enter UAR verification.
The retained failure confirms the conflict classification. The attribution to
the helper's raw signing order is a source inference consistent with the
existing initialization pattern and `.prometheus/gotchas.md:234–251`; no new
upstream behavior claim or independent reproduction is asserted here.

## Proposed minimum repair after the lead closes the running program

Assign `tests/support/credential_process.rs` to one writer. In
`authenticated_context`, before calling `token`, invoke
`authenticate_bearer_context(&config, "not-a-token").await` and assert that its
error is `axum::http::StatusCode::UNAUTHORIZED`. Then sign and authenticate the
real synthetic token as before. This follows the existing fixture initialization
mechanism while making initialization failure explicit rather than ignoring it.
Use `expect_err` followed by the status assertion; `UserContext` has no
`PartialEq` implementation, so do not compare the whole `Result` with `assert_eq!`.

Do not expose the crate-private JWT helper, accept an already-installed unknown
provider, synthesize `VerifiedIdentity`, or change middleware's status mapping.
Those would alter a real host boundary to accommodate fixture setup. No public
interface, stored credential key, tenant binding, pin, or vendor change is needed.

The proposed assertion traces directly to the observed 500 and to UAR's
process-provider ownership boundary. No speculative guard is proposed.

## Downstream source context and unresolved limits

`src/embedded.rs:463–474` passes the supplied driver and credential policy into
the run manager. `src/uar/runtime/manager.rs:5408–5414` rejects an opaque supplied
driver under shared policy before `RunModelBindings::capture`; the allowed path
selects the supplied driver in `src/uar/runtime/turn/bindings.rs:266–268`.
These are source facts, not observed successful SDK execution. The retained run
does not verify either the single-owner provider contact or the shared-policy
zero-contact assertion at `tests/credentials_api_integration_test.rs:773–780`.

The helper's missing signed `uar_credential_kind` is not a reason to add a marker
for this failure: credential-owner issuer classification uses the verified issuer
at `src/uar/security/claims.rs:199–219`, and the adapter rejects API-key provenance
but does not require issuer privilege (`middleware.rs:119–126`). This scenario
does not request trusted-host service authority. Adding authority would broaden
the fixture unnecessarily.

The uncomfortable limit: fixing first failure ordering can reveal another
downstream failure. No source review can certify the provider/store outcome or
prove that this is the final required repair. The lead must include this change
in the complete repair inventory and rerun the failed gate serially only after
the phase implementation boundary is reopened for verification.

Research used repository source, retained local command evidence, phase plans,
the team repository map/tool policy/skill policy, and the named security/Rust
skills. The map describes an older Compass graph with parse/coverage gaps; no
graph edge or stale line reference was treated as runtime evidence. Commands
were limited to `pwd`, `cat`, `sed`, `nl`, `rg`, `head`, and `tail` source/evidence
reads plus the patch that created this file. Some exploratory reads reported
nonexistent candidate paths before the actual module directories were located;
these were source-navigation misses, not product failures.

Verification commands run by this diagnostic agent: **none**. Tests, builds,
compiler probes, lint, formatting, validators, imports, HTTP calls and external
gates were deferred per the assignment. No unsolicited implementation or guard
was added. Independent acceptance and the proposed fixture repair remain pending.

## R8 implementation addendum — 2026-10-16

The lead reported the sole acceptance program finished with exit 1 and closed the
full implementation boundary for this repair cohort. At repair start, waypoint
revision 3009 still identifies Execute, task `6.2`. The lead assigned exclusive
ownership of the two source files below and this handoff. This addendum supersedes
the initial diagnosis's pending-implementation status; acceptance remains pending.

Exact changed paths and effects:

- `tests/support/credential_process.rs`: added private async
  `initialize_jwt_verifier` at line 250. It invokes the existing public
  `authenticate_bearer_context` with malformed input, requires an error through
  `expect_err`, and asserts the status is exactly `UNAUTHORIZED`. The SDK
  `authenticated_context` invokes it at line 482 before token signing. The two
  pre-existing initialization calls in `Node::boot` (line 130) and
  `authenticated` (line 287) now use the same helper instead of discarding their
  results. All three configurations require JWTs. A 500 initialization conflict
  or 503 identity-policy failure therefore stops fixture setup explicitly.
- `src/uar/api/credentials.rs`: added an explicit `std::fmt::Debug` implementation
  for `CredentialApiState` at line 11. Its sole field is the boolean
  `service_available`, obtained through `Option::is_some`; it finishes as
  non-exhaustive. It never formats the policy, service, credential material,
  mappings, issuer/tenant identities, store, or encryption internals. This adds
  the Debug trait interface to address the observed missing-Debug warning.
  HTTP response and authorization behavior are unchanged by this implementation.
- `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/coordination/handoffs/R8-sdk-diagnosis.md`:
  retained diagnosis and added this exact implementation and verification handoff.

Source inspection after editing used `nl -ba` with `sed` for the modified spans
and `rg -n 'token\(|initialize_jwt_verifier|authenticate_bearer_context'
tests/support/credential_process.rs`. The displayed source places initialization
before both token-producing authentication calls and shows all three setup paths
using the helper. The raw `token` utility remains synchronous; callers must still
initialize UAR first through the fixture setup. No broader token API change was
made. These observations are source reasoning, not compile/runtime results.

The initialization assertion addresses the observed first-acquisition conflict
and preserves the real trusted-host JWT provider boundary. Debug's restricted
output addresses the observed warning at a credential-bearing state boundary.
No unrelated guard, production policy change, dependency edit, vendor edit or
unrequested artifact was added. Existing user and teammate edits were preserved.

Verification commands run for this repair: **none**. No tests, Cargo, compiler
probes, builds, lint, formatting, validators, imports, runtime calls or independent
review ran. Per the lead's assignment, all remain deferred until the renewed L6
inventory covers the entire cohort. Compilation, warning elimination in the
actual gate, single-owner provider success, shared-policy zero-contact denial,
and any downstream failure revealed after initialization are unverified. There
are no unresolved design decisions within this assigned repair.
