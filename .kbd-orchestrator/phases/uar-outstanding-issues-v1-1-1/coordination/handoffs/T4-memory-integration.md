# T4 memory integration supplement

Phase `uar-outstanding-issues-v1-1-1`, Execute. Source authoring complete for
this bounded assignment. Full-phase verification remains CLOSED. No test,
Cargo invocation, compiler probe, build, format, lint, validator, import probe
or independent review ran. The lead owns the remaining inventory and gate.

## Changed paths

- `tests/credentials_api_integration_test.rs`: register one support module and
  exactly one additional nonignored integration case,
  `phase_memory_credential_migration_uses_real_http_and_atomic_store`.
- `tests/support/uar_1_1_1_memory_fixture.rs` (new): actual loopback HTTP fixture
  and the supported process-local memory migration contract.
- `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/coordination/handoffs/T4-memory-integration.md`
  (new): this source handoff.

Production code, dependency pins, manifests, existing adapters and live systems
are unchanged. No public API or provider wire contract changes. No unrequested
guards or production hooks were added.

## Real path and source-grounded scenario

The fixture binds an ephemeral `127.0.0.1` TCP listener and sends actual
`reqwest` HTTP requests to the production `credentials::build_router`.
Its ingress adapter calls production `authenticate_bearer_context` for each
request's Authorization header. The configured issuer, audience, expiry,
signature and signed `uar_credential_kind: issuer` marker are verified before
inserting the returned proof. Exact trusted-host configuration enables preview;
the request cannot construct `HostAuthority` or a synthetic `UserContext`.
The router receives a real `ProviderService`, `CredentialTenancyPolicy` in
shared mode, actual encryption and `InMemoryCredentialStore`.

Source basis: `CredentialStore::put` always assigns a fresh revision, even when
the supplied timestamps/ciphertext are identical. `migration_preview` derives
the exact source/destination/approval precondition. `migrate_legacy_conditioned`
requires an installed exact mapping; the store compares the source revision
and commits destination plus immutable journal under one state write lock.
The executable scenario is authored to detect stale-copy acceptance, duplicate
journals, destination overwrite or resurrection at those actual boundaries.
This is a source-grounded reproducer/acceptance artifact, not an observed failing
runtime result. No production fix was made by this assignment.

Authored observations:

- Missing bearer and forged signature return 401. A verified ordinary owner
  cannot preview (403); same subject in another tenant cannot apply or read the
  destination. Cross-tenant apply remains denied after the journal exists.
- Preview returns one exact approved source, its revision and precondition,
  safe metadata/counts, and no ciphertext, hint or free-form receipt reason.
  A direct store read confirms preview leaves source metadata/ciphertext intact.
- Rewriting the source with identical timestamps/ciphertext changes revision;
  the old tag returns 412 with no destination or journal.
- Eight concurrent real HTTP requests with the fresh tag return identical
  receipts; actual inventory contains one journal and one copied destination.
  Copy preserves ciphertext and timestamps while recording destination revision.
- HTTP rotation preserves creation time and replaces the secret/revision. Replay
  with a changed request reason returns the original immutable receipt and does
  not overwrite the rotated destination. Preview reports destination changed.
- HTTP deletion followed by exact retry returns the original receipt, leaves
  the destination absent, and reports destination missing. Removing the source
  through the real store and retrying still returns that receipt without
  resurrection. The original stale tag stays rejected. Inventory retains one
  byte-equivalent serialized metadata journal and zero credential rows.
- Full synthetic secret canaries never appear in HTTP responses. Ciphertext
  assertions avoid printing secret-bearing values on failure. Tokens and keys
  are generated solely for this isolated fixture. The HTTP client disables
  ambient proxy use and has a bounded request timeout. The listener task is
  aborted/joined on successful completion and aborted on unwinding.

## Exact future command

The lead owns execution and the existing serialized target directory. This
command is supplied for registration only and has NOT run:

```sh
CARGO_TARGET_DIR=target/uar-1-1-1-phase cargo test --features server-full,postgres-backend,in-memory-backend --test credentials_api_integration_test phase_memory_credential_migration_uses_real_http_and_atomic_store -- --exact --test-threads=1
```

No external database, provider, container, ambient credential or live know-me
resource is used by this case. The containing target retains its existing
`server` + `surreal-backend` compilation guard; the registered feature set
includes those features. Executing this exact name does not run existing cases.

## Unverified limits and handoff decisions

The uncomfortable boundary is scope: this is production issuer-JWT verification
and credential-router HTTP integration, but its small ingress adapter is not
the full `AppState` `auth_middleware`. It proves no API-key revocation,
delegation grant, workspace admission, installed binary or restart behavior.
Those paths belong to the existing production-image/native suites.

Public memory writes always initialize revisions; the map state is private and
there is no supported historical-row or fixed-local raw-owner seed API.
Process-local memory has no historical persisted state to upgrade across
invocations. `credentials::maintenance::initialize_revisions` explicitly requires
a durable backend. Therefore this case does not fabricate missing revisions,
fixed-local rows or a maintenance CLI lifecycle. The lead accepted that boundary;
actual KV/Surreal/PostgreSQL adapters cover those historical contracts. No new
production test hook or unchecked identity constructor was introduced.

Source inspection used `cat`, `sed` and `rg` against the instructions, T3/T4
handoffs and production contracts. These are source evidence only; no fresh
graph result or runtime result is claimed. All Rust syntax, feature-dependent
compilation, actual HTTP statuses, concurrent transaction behavior and cleanup
remain uncompiled/unexecuted pending the full-phase real integration gate.

No outstanding design decision remains for this bounded supplement. The lead
must register the exact command and retain the stated coverage limits in L4.
