# State execution packet — S1 through S4

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Plan. Owner: `uar-state`.

This packet turns only stable OpenSpec task identities `S1` through `S4` into
state implementation work. It does not change their selected scope, authorize a
database operation, migration execution, fixture invocation, product check,
build, formatter, validator, or review. The full-phase boundary remains CLOSED
until L4 records every implementation item in the whole phase.

## Authority, evidence, and non-goals

The authoritative waypoint names this phase as ready for Plan. The source
contracts are `spec/state-contracts.md`, especially the safe captured-store
eligibility and S3 handoff, and the T3 schema sections of
`spec/trust-contracts.md`. `openspec/.../tasks.md` and `verification.md` keep
the stable task IDs and set the final evidence boundary.

Source inspection establishes these current interfaces:

- `PersistenceLayer` currently offers boolean durable-accounting and
  durable-ingestion capabilities. `SurrealDbProvider::new_with_auth` captures
  the remote-attestation environment value while it constructs the provider;
  it then stores only `durable_instances` and
  `remote_requires_durability_attestation`.
- The knowledge route rejects non-durable stores before upload acceptance, and
  the dispatcher preserves source/job state through discovery, compare-and-swap
  transitions, and reconciliation. Principal knowledge tables are already
  disjoint from legacy tables.
- `MemoryService` is a facade over the vendored `surreal-memory` dependency and
  already requires verified owner binding through `OwnedMemory`. It is not a
  credential store, a KB-import path, or a substitute for UAR persistence.
- PostgreSQL discovers additive SQL files through `sqlx::migrate!("./migrations")`;
  the Surreal host explicitly registers each `.surql` file from
  `surreal_host.rs`. Existing principal credential rows and receipt tables are
  provisioned by `principal_credentials.sql` and
  `principal_credentials.surql`.

No task adds a persistence-health endpoint, re-reads mutable environment per
request, treats connectivity as durable backing, imports or adopts legacy KB
data, revives a cancelled Dot run, forces team takeover, or changes exact
checkpoint/tool/accounting receipts. Existing uncertain writes remain reserved
or pending until their present reconciliation procedures reach a durable state.

## Serialized ownership and dependency order

| Order | Stable task | Writer and exclusive paths | Inputs / required handoff | Completion boundary |
| --- | --- | --- | --- | --- |
| 1 | S3 | `uar-state`: NEW `migrations/20261016000000_credential_revisions.sql`; NEW `migrations/surrealdb/credential_revisions.surql`; existing `src/uar/persistence/providers/surreal_host.rs` | T3 supplies the already-selected row, receipt, precondition, and journal contract; state supplies additive physical schema only. | State hands schema names, nullable-read compatibility, initialization rules, and backend differences to the sole T3 credential-store writer. |
| 2 | S1 | `uar-state`: existing `src/uar/persistence/mod.rs`, `src/uar/persistence/providers/surreal.rs`, `src/uar/persistence/providers/surreal_host.rs`, `src/uar/persistence/providers/postgres.rs`, and `src/uar/persistence/providers/memory.rs` | Runtime supplies its safe admission-stage projection; state exposes no runtime error formatting or public receipt shape. | Runtime receives a typed captured value, never an endpoint, namespace, credential, mutable environment read, or raw error. |
| 3 | S2 | `uar-state`: existing `src/uar/api/knowledge.rs`, `src/uar/rag/ingestion_worker.rs`, `docs/operations/durable-ingestion.md`, and `docs/operations/knowledge-principal-isolation.md` | Runtime gives reviewed wording for its separate run-admission diagnostic. Trust supplies only existing qualified recovery terminology. | The upload and dispatcher project the same safe category without changing the `202` atomicity contract or adding a recovery authority. |
| 4 | S4 | `uar-state`: existing `tests/durable_ingestion.rs`, `tests/principal_knowledge_namespace.rs`, and NEW `tests/support/uar_1_1_1_state_fixture.rs` if shared state fixture setup cannot remain local to those two targets | L3 owns runner, manifests, receipt paths, Docker image orchestration, and one build target. Runtime and trust hand their scenario adapters to L3. | Fixtures and adapter inputs are authored only. They remain unexecuted until L4, V1, and the selected V2/P2 groups. |

`src/uar/persistence/providers/surreal_host.rs` is the only existing state
file with two state responsibilities in this packet: register the new Surreal
schema exactly once and construct an embedded host provider with its captured
eligibility. S3 precedes T3 implementation. S1 precedes R1 integration. S2
depends on S1's trait handoff. S4 depends on the finalized S1/S2/S3 contracts
but does not permit execution.

The trust role is the **sole writer** of `src/uar/security/credentials/store.rs`
and `src/uar/security/credentials/principal_store.rs`; state does not edit those
files. The lead owns shared startup/CLI/configuration integration, including the
proposed explicit initialization command and any `src/main.rs`, `src/config.rs`,
or `src/server.rs` changes. No concurrent writer edits those roots.

## S1 — captured durable-store eligibility

### Planned changes

1. In `src/uar/persistence/mod.rs`, add one host-private, typed captured
   eligibility enum and one trait getter. Its only variants serialize or map to
   the exact safe vocabulary `local_durable`, `remote_attested`,
   `remote_not_attested`, and `unsupported_backend`. It is a store fact, not a
   run outcome, accounting result, health API, or formatted error.
2. In `src/uar/persistence/providers/surreal.rs`, replace the
   boolean-only explanation retained at construction with the enum captured
   from the normalized endpoint class and the one-time trusted-host attestation
   input. Keep `durable_instances` behavior and its accounting/ingestion gates
   aligned with the captured eligible variants. Do not read
   `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` after construction.
3. In `src/uar/persistence/providers/surreal_host.rs`, have the trusted embedded
   RocksDB/SurrealKV constructor report `local_durable`. Its existing migration
   registration must stay in the host path; it must not create another client or
   reinterpret a caller-selected namespace/database.
4. In `src/uar/persistence/providers/postgres.rs`, report the currently
   supported durable provider as `local_durable` for this capability. In
   `src/uar/persistence/providers/memory.rs`, report `unsupported_backend`.
   Neither change adds remote-attestation configuration for a non-Surreal
   provider.

### Interface handoff to runtime

`PersistenceLayer` gains only a read-only typed eligibility getter. R1 maps it
to its separately specified allowlisted accounting/admission category and opaque
diagnostic reference. The runtime preserves existing
`orchestrator_start_failed`, AG-UI `RUN_ERROR` code/message shape, refusal
ordering before provider client/tool effects, verified owner partitioning, and
unreconciled liabilities. State owns no run-manager or diagnostic receipt file.

The uncomfortable case is a remote endpoint that connects and accepts writes
but loses state after restart. It remains `remote_not_attested`; neither a
successful connection, HTTP health response, replica count, nor writable user
can change that classification.

## S2 — durable ingestion, reupload, and bounded recovery guidance

### Planned changes

1. In `src/uar/api/knowledge.rs`, use S1's captured safe eligibility when the
   existing durable-ingestion precondition refuses an upload. Keep the existing
   status and public message unless the runtime handoff explicitly owns a
   compatible response projection. The refusal remains before source bytes,
   document, and job commitment; eligible uploads still atomically create all
   three before returning `202`.
2. In `src/uar/rag/ingestion_worker.rs`, use the same captured category in the
   existing dispatcher-disabled/reconciliation diagnostic path. Queue loss,
   compare-and-swap conflict, missing source bytes, and interruption preserve
   their current durable discovery and reconciliation behavior. The worker does
   not manufacture a retry or mark an unknown write successful.
3. `docs/operations/durable-ingestion.md` receives the state-owned procedure:
   establish operator-attested backing; capture verified principal and selected
   namespace/database identity; inventory legacy/principal rows read-only;
   create or reuse a principal KB; reupload retained source bytes; wait for the
   durable job to index; retrieve under the same principal. It says missing
   original bytes require a new source and that legacy rows are retained.
4. `docs/operations/knowledge-principal-isolation.md` records that matching old
   subject/document/chunk/job identifiers do not prove owner identity, so a
   reupload makes new principal-scoped records and never adopts legacy rows.
   It also distinguishes storage eligibility from separate provider/model
   reverification.

Documentation may reference the established recovery proof cases only:
authenticated quiesce/reclaim with its exact joined receipt, or qualified
trusted-supervisor exclusion with its single-use evidence. It must state that
active effects and accounting remain uncertain for reconciliation and recovery
is blocked without qualified proof. It must reject a force operation, timeout,
deletion, Pod absence, and replica count as takeover evidence. State does not
add recovery implementation or an import API.

## S3 — additive credential-revision schema handoff

### Schema files and preservation rules

State adds the paired physical schema revisions below; both are additive and
retain historical rows and receipts.

| Store | New file | Required additive shape |
| --- | --- | --- |
| PostgreSQL | `migrations/20261016000000_credential_revisions.sql` | Add nullable `row_revision` metadata to the supported credential source/destination rows and allow the journal's existing JSONB receipt to carry additive source/destination revision and precondition metadata. It must not rewrite ciphertext, hint, created/updated timestamp, owner key, legacy source key, or an existing receipt. |
| SurrealDB | `migrations/surrealdb/credential_revisions.surql` | Define additive schemaless-safe fields/indexes needed to read the same optional revision metadata and preserve the current unique principal credential and immutable migration-journal identities. No `DELETE`, `REMOVE`, broad update, or historical rewrite is selected. |

`src/uar/persistence/providers/surreal_host.rs` registers the Surreal file
serially with its existing migration list. PostgreSQL needs no provider registry
edit because it discovers SQL migration files from `./migrations`. The changes
must target both the legacy source surface and principal credential surface
selected by T3, while preserving ordinary historical reads when `row_revision`
is absent. The T3 writer owns Rust record/store projections and all write paths:
every newly accepted write or recreation receives a fresh UUID revision;
rotating or deleting a destination never rewrites historical migration receipts.

### Explicit initialization, not startup backfill

The schema supports an explicit trusted-host maintenance operation proposed by
trust: `universal-agent-runtime credentials initialize-revisions
--writers-quiesced`. The lead owns command/configuration wiring; trust owns the
maintenance implementation and exact store operation. State's interface handoff
requires all of the following:

- It runs only after backup and actual writer quiescence. The acknowledgement
  flag records operator intent but is not proof of exclusion.
- It initializes missing revisions while preserving ciphertext, hints, owners,
  timestamps, and existing receipts; repeated invocations leave assigned
  revisions unchanged and return safe counts only.
- Startup, ordinary CRUD, `/migration-preview`, and source discovery never
  backfill a revision implicitly. The command opens no listener and contacts no
  provider or MCP service.
- Migration execution later compares an exact source revision and destination
  vacancy/receipt atomically. A lost response is reconciled by retrying the
  exact approved operation against the journal; it is never treated as a new
  source or permission to overwrite.

T3 consumes optional legacy revision fields to report `revision_unavailable`
rather than inventing a value. New receipts use the selected additive metadata
(`source_kind`, `source_owner_key`, `source_revision`,
`destination_revision`, `precondition`) while legacy three-element source keys
and historical receipt fields remain byte-compatible. The schema is not a
general principal migration and does not modify session, schedule, followup,
knowledge, approval, or durable-work identities.

## S4 — fixture and receipt inputs

S4 authors only fixture source and adapters. L3 owns the new acceptance runner,
the three manifests, run-root lifecycle, receipt paths, image build/pull logic,
and serial execution.

`tests/durable_ingestion.rs` will extend its real durable-store child-process
fixture to cover: unattested remote Surreal refusal before `202` and before
source/job creation; attested remote commit of source/document/job; dispatcher
rediscovery through a joined restart; same-owner retrieval; and the retained
liability/error path. The fixture continues to use a separate process per boot,
not a second service over one live provider handle.

`tests/principal_knowledge_namespace.rs` will extend its durable Surreal and
PostgreSQL setup to seed a legacy subject-like record, show that it remains
invisible to the verified owner, and prove a new reupload creates/readbacks only
principal-scoped data. A second principal and different issuer/tenant variant
must receive no document, job, retry, or search result. No fixture changes a
legacy namespace marker to call it migrated.

If shared private infrastructure parsing or authenticated setup must be reused
by both targets, state creates only
`tests/support/uar_1_1_1_state_fixture.rs`; otherwise no new support file is
created. It consumes L3's private infrastructure description without writing
credentials to source, manifests, logs, or receipts. The L3 handoff must bind
candidate/published run evidence to source/fixture identity, configured
SurrealDB image digest, authenticated principals, owned scratch namespace and
volume, true process restart, observed HTTP status, retrieval result, denial,
and redacted receipt hashes.

These fixtures supplement the required final `recovery` and `combined` groups;
direct trait calls, mocks, synthetic receipts, or an in-memory-only result never
satisfy acceptance. Fixture authoring does not authorize their execution.

## Final handoffs and unverified state

Before Execute, state hands the lead the exact file inventory above and the
following dependencies: S3 schema registration before trust T3; S1 trait type
before runtime R1; S1/S2 fixture adapters before L3's final runner integration;
and runtime wording before state finalizes the state-owned durable-ingestion
documentation. The lead serializes any shared-file edit and records all source
identities in L4.

Unverified by design: the new schema syntax against the configured SurrealDB,
the full PostgreSQL/Surreal revision migration, explicit initialization,
credential precondition and replay behavior, remote volume durability, joined
restart, knowledge upload/retrieval, owner isolation, and release-image
receipts. No command was run for any of them. `versions.toml` currently pins
SurrealDB and its image at `3.3.0`; the role request's reference to `3.2.4`
conflicts with that authority and must be resolved by the lead before a
version-specific migration or fixture assertion is accepted.


## Lead consolidation — 2026-10-16

The authoritative sequence in plan.md is S1, then S3 after the T1 interface,
then S2, then S4; the table above groups responsibilities rather than imposing
a conflicting global order. S3 owns surreal_host.rs registration and S1 owns its
eligibility change under the same sole state writer. The existing Spec resolves
the stale role pin: use versions.toml SurrealDB 3.3.0; no unresolved version
decision remains. S1 is self-contained and does not wait for R1 formatting; it
hands captured eligibility to runtime. S2 supplies its own safe upload category
and passes draft text to runtime, avoiding a reverse runtime dependency.

S4 also owns NEW scripts/release/uar_1_1_1_recovery.py. It implements the L3
run_group(context) interface for installed-image upload/retrieval/restart and
existing qualified recovery proof controls (valid, absent, forged and replayed),
using actual runtime APIs and durable state. It does not introduce a production
force-recovery path. R5 receives setup/assertion helpers for combined coverage;
L3 owns lifecycle and receipt writing. This adapter and its documentation are
required before L4, not deferred test implementation.
