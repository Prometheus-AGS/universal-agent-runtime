# Credential cohort repair — 2026-10-16

Phase: `uar-outstanding-issues-v1-1-1`, Execute, V2 corrective cohort.
Writer: `credentials_cohort` / `uar-trust-tools`.
Source identity at inspection: HEAD `91a091d647fe2ce63651c07140172aba2c6c6302`;
shared dirty checkout with existing phase implementation. HEAD alone does not
identify that implementation. This handoff records only this writer's delta.

## Observed failure and source-grounded reproducer

Retained evidence:
`evidence/acceptance/20261016-local-02/command-00019.json` and `.log`, relative
to the phase directory. The already-run command was:

```text
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test credentials_api_integration_test phase_memory_credential_migration_uses_real_http_and_atomic_store -- --exact --test-threads=1
```

Observed existing output: exit 101, `0 passed; 1 failed`, panic at
`tests/support/uar_1_1_1_memory_fixture.rs:163:84`, `Err` value `500`.
The failing call was initial owner authentication, before HTTP server setup or
credential writes. This command was read from retained evidence, not rerun.

Source recipe: select the memory scenario alone in a fresh process. Its
`conditional_migration` originally called `bearer`, which calls raw
`jsonwebtoken::encode`, before `authenticate_bearer_context`. In contrast,
`credential_process::{Node::boot,authenticated,authenticated_context}` already
initialize UAR's verifier before fixture signing. `src/uar/security/jwt.rs`
records failed process-provider installation as `ProviderConflict`, and
`src/uar/security/middleware.rs` maps that variant to 500. The observed failure
and this source ordering identify the omitted fixture initialization. No claim
about a successful repaired runtime or durable migration follows from this trace.

The uncomfortable case is that repairing setup can expose later migration
failures. The failed command reached none of its migration assertions.

## Changed paths and interface effects

- `tests/support/credential_process.rs`: expose existing
  `initialize_jwt_verifier` as `pub(super)` within the integration test's module
  tree. Its implementation still submits a malformed token to production
  authentication and requires 401. Production APIs are unchanged.
- `tests/support/uar_1_1_1_memory_fixture.rs`: call that helper before the first
  `bearer` call. Retain the existing HTTP ingress, verified owner construction,
  foreign-tenant denial, source revision drift, concurrent atomic migration,
  source retention, immutable retry, rotation/deletion, and secret-canary
  assertions.
- `docs/operations/principal-provider-credentials.md`: document initialization
  ordering and the limits of the retained failure. Lead explicitly assigned this
  documentation path during this task.
- This handoff file: source diagnosis, ownership delta and deferred evidence.

No production change is justified by this pre-storage failure. The actual
`InMemoryCredentialStore` source retains a common state lock for legacy rows,
principal rows and migration journals; migration checks the bound destination,
revision and vacancy before inserting destination and journal under that write
lock. Credential API identity and trusted-host preview checks remain in place.
These are source observations, not runtime results. No MCP metadata grants,
kernel writes, dependency pins or vendored sources were changed.

## Verification and remaining limits

No test, check, compiler, lint, format, build, probe or independent review command
was run for this repair. Repository full-cohort instructions defer all such work
until production, fixtures, documentation and supervisor implementation are
complete and the lead records the boundary. Only file reads/searches and patches
were performed. The helper visibility and repaired execution remain uncompiled
and unexecuted in this task.

`command-00018.json` records the previous SDK scenario as passed, exit 0. This
repair neither reruns that scenario nor rewrites that receipt. It is historical
evidence for its original source state, not acceptance of this delta.

No unsolicited feature or new production guard was added. The reused malformed-
token assertion traces directly to the observed provider-initialization failure;
its existing rejection requirement remains intact. No unresolved implementation
decision remains in this assigned repair. Full-cohort acceptance and any subsequent
runtime failure investigation remain with the lead after the implementation
boundary, using the preserved local-02 evidence.
