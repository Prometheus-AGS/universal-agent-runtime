# State contracts and production acceptance

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Spec. Date: 2026-10-16.

This contract defines future production acceptance. It authorizes no database
operation, migration, fixture execution, build, test, lint, review, release, or
deployment operation during the closed phase boundary.

## State interface contracts

### S1 — captured durable-store eligibility

`PersistenceLayer` SHALL expose one additive host-private eligibility value that
is captured when the provider is constructed. Its safe vocabulary is exactly
`local_durable`, `remote_attested`, `remote_not_attested`, and
`unsupported_backend`. It is a captured store fact, not an accounting outcome:
verified-principal requirements and reconciliation states remain accounting
outcomes. The value is never a formatted error and excludes endpoint URLs,
namespace/database names, users, credentials, source errors, and mutable
environment reads.

`SurrealDbProvider` captures remote attestation once. Existing durable
accounting and ingestion gates retain their decisions from that captured state.
The state-to-runtime handoff is the typed captured store eligibility.
`uar-runtime` generates an opaque diagnostic reference at refusal and maps the
eligibility to its allowlisted accounting stage/category
(including unsupported-backend and verified-principal outcomes) while preserving
`orchestrator_start_failed`, AG-UI `RUN_ERROR` code/message shape, and refusal
before provider or tool effects. This contract does not add a persistence-health
API.

Verified owner partitioning remains mandatory. Existing checkpoint,
tool-admission, accounting, and ingestion receipts remain immutable. An
ambiguous write retains its exact reservation or work item until the existing
reconciliation procedure determines its state; restart uses the same owner and
durable store rather than a process-local ledger.

### S2 — durable ingestion, reupload, and qualified recovery documentation

`POST /api/knowledge/{id}/documents` remains the upload entry point, also
mounted at `/api/uar/knowledge-bases/{id}/documents`. Its `202 Accepted`
contract remains atomic commitment of source bytes, owner-scoped document, and
one ingestion job. Eligibility refusal occurs before that response.
`GET /api/knowledge/{id}/documents/{doc_id}/ingestion` and
`POST /api/knowledge/{id}/search` remain the observable job and retrieval
paths. The dispatcher uses the same captured safe category and the existing
reconcile/discover/transition/publication path.

Authenticated reupload requires retained source bytes, the verified current
principal, and an operator-qualified durable profile. It creates new
principal-scoped records and leaves legacy rows unchanged. A matching legacy
subject, document, chunk, job, or namespace is not owner evidence. Missing
source bytes are a documented recovery limit.

Fenced team recovery remains documentation for only the proofcases already
specified by `openspec/specs/dot-supervisor-recovery/spec.md`: authenticated
quiesce/reclaim and qualified trusted-supervisor exclusion. Documentation SHALL
require the exact reclaim receipt, preserve uncertain active effects and
accounting for reconciliation, and declare recovery blocked without qualifying
proof. It SHALL NOT add a force operation, timeout, deletion, Pod-absence, or
replica-count takeover path. This contract does not revive or qualify cancelled
Dot work.

### S3 — credential-policy schema handoff to T3

T3 receives this additive contract for `tenant-isolation`. The precise policy,
grant, row-revision, mapping and HTTP schemas in [trust-contracts.md](trust-contracts.md)
are authoritative for this summarized handoff. It SHALL NOT change
generic principal storage keys or unrelated session, schedule, followup,
knowledge, approval, or durable-work identities.

| Additive contract | Required fields and limits |
|---|---|
| `CredentialTenancyPolicy` | `mode` is exactly `single_owner` or `shared`; omission resolves to `single_owner`; remote transport does not select the mode. |
| `CredentialOwnerBinding` | A verified principal plus credential-only owner derivation. Issuerless/local-host derivation requires a stable operator-provisioned installation UUID. It survives restart/replica replacement and is never accepted from a request, session, or Pod identity. Externally issued verified principals keep their existing derivation. |
| `CredentialFallbackGrant` | In `shared` mode, house/system fallback requires an explicit operator grant bound to the verified owner and governs primary, missing-service, inherited-client, and fallback-provider capture paths. Missing ownership or grant rejects before client capture. |
| `CredentialMigrationPreview` | Trusted-host-only, non-secret inventory of source/destination identifiers, counts, missing mappings, existing destinations, and conflicts for legacy and fixed-local v2 sources. It emits no ciphertext, credential value, raw subject fallback, or ownership adoption. |
| `CredentialMigrationReceipt` | An approved exact mapping and source identity bind preview to execution. Copy-plus-journal behavior preserves source rows, detects source change/conflict, remains idempotent for the same mapping, and never reconstructs overwritten history. |

The existing credential surface is mounted at `/api/uar/credentials`, including
`GET /principal` and `POST /migrate-legacy`. T3 SHALL add the trusted-host-only
`GET /migration-preview` and authenticated-destination `POST /migrate-principal`
interfaces specified in trust-contracts.md. Migration requires the exact installed
operator mapping; it does not require a trusted-host caller. Neither interface
widens ordinary CRUD or exposes secrets. Execution requires backup, writer quiescence, exact mapping
approval, count verification, cold readback, and the documented rollback limit:
a subject-only binary cannot safely retain shared access.

### S4 — final-boundary fixtures and receipts

Final evidence SHALL use actual UAR routes, configured SurrealDB 3.3.0,
authenticated principals, the durable ingestion worker, and the
installed/candidate runtime. Direct trait calls, mock-only checks, and
source-only assertions are supplemental and cannot satisfy acceptance.

## Behavior: bounded remote-store admission

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1 / S1` |
| Source identity | Candidate revision and configured SurrealDB 3.3.0 image digest in the receipt. |
| Production entry point | Authenticated `POST /api/uar/runs` selecting a run with an explicit token or cost limit, followed by its stream/status observations. |
| Real collaborators | UAR server, `RunManager`, `PersistenceLayer`, configured remote SurrealDB, authenticated principal, selected provider route, and configured tool registry. |
| Boundary exercised | HTTP admission and remote durable-accounting eligibility before provider-client construction and tool execution. |
| Observable result | Unattested remote profile returns the existing public admission failure with only allowlisted category/reference projection; attested profile creates durable owner-scoped accounting that survives joined restart. |
| Negative control | Same remote endpoint without trusted durability attestation; provider and tools observe no invocation and no accounting success receipt. |
| Isolation | Runner-created scratch configuration, database namespace/database, credentials, ports, logs, and cleanup owner; no live UAR root or operator database. |
| Prerequisites | Full production inventory, disposable remote SurrealDB 3.3.0, authenticated test principals, candidate runtime, and no unfinished state contract. |
| Final local gate | `python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local.json` |
| Evidence | Future receipt under the local manifest’s `evidence_root` with source identity, image identity, endpoint observations, negative control, exit code, and receipt hashes. |
| Limitations | Does not attest the reported deployment's storage volume or restore historical state. |

## Behavior: authenticated knowledge reupload and dispatcher recovery

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1 / S2` |
| Source identity | Candidate revision, retained fixture source-byte digest, and SurrealDB 3.3.0 image digest in the receipt. |
| Production entry point | Authenticated `POST /api/knowledge/{id}/documents`, then `GET /api/knowledge/{id}/documents/{doc_id}/ingestion` and `POST /api/knowledge/{id}/search` after joined restart. |
| Real collaborators | UAR HTTP server, `KnowledgeApiState`, `PersistenceLayer`, durable ingestion worker, vector/embedding collaborator, remote SurrealDB, and two authenticated principals. |
| Boundary exercised | Multipart upload, atomic durable source/document/job commitment, dispatcher reconciliation, process restart, owner-scoped retrieval, and principal/legacy isolation. |
| Observable result | Eligible upload receives `202`; exactly one owner-scoped job indexes and retrieves after restart. Legacy rows remain unchanged and another principal receives no matching document, job, retry, or search result. |
| Negative control | Unattested remote profile rejects before `202`; a second principal and seeded legacy subject-like row cannot read or adopt reuploaded content. |
| Isolation | Runner-created source fixture, credentials, database namespace/database, vector data, ports, logs, and cleanup owner. |
| Prerequisites | Full production inventory, retained source bytes, disposable durable profile, authenticated principals, and restartable candidate runtime. |
| Final local gate | `python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local.json` |
| Evidence | Future receipt under the local manifest’s `evidence_root` records source digest, identities, job/document identities, restart result, denial result, exit code, and receipt hashes. |
| Limitations | Missing source bytes cannot be recovered; no legacy import or customer ownership proof follows. |

## Behavior: credential policy and migration safety

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1 / S3 / T3` |
| Source identity | Candidate revision, installation-UUID identity class, and non-secret migration-preview receipt. |
| Production entry point | Authenticated `/api/uar/credentials/principal` and destination-authorized migration with the exact installed operator mapping; trusted-host-only `/migration-preview`. |
| Real collaborators | UAR HTTP server, verified security context, credential resolver/principal store, encryption service, trusted installation configuration, and durable credential store. |
| Boundary exercised | Verified-principal credential derivation, shared fallback authorization, fixed-local preview/execution, and cold readback after restart. |
| Observable result | Omitted policy is single-owner; shared fallback needs explicit grant; missing local installation identity rejects affected work; external issuer credentials remain compatible; preview/execution bind one source identity. |
| Negative control | Unverified/mismatched principal, missing shared grant, missing local UUID, changed source after preview, and conflict each refuse without client capture, secret output, or source overwrite. |
| Isolation | Runner-created encryption key, credential store, installation UUID, principals, database namespace/database, ports, logs, and cleanup owner. |
| Prerequisites | Completed T3 implementation, approved exact mapping, backup, writer quiescence, and candidate runtime. |
| Final local gate | `python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local.json` |
| Evidence | Future receipt under the local manifest’s `evidence_root` with non-secret preview, mapping identity, cold-readback, denials, exit code, and receipt hashes. |
| Limitations | Cannot prove subject-only rollback preserves shared access or repair non-credential identities. |

## Behavior: immutable multi-architecture release evidence

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1 / S4` |
| Source identity | Immutable v1.1.1 source commit, tag, publication workflow/run, manifest digest, and arm64/amd64 child digests. |
| Production entry point | Installed published image executes S1-S3 production operations against isolated collaborators. |
| Real collaborators | Published arm64 and amd64 UAR images, matching remote SurrealDB profiles, authenticated principals, release manifest, and declared native/emulated execution host. |
| Boundary exercised | Pull/install, startup, HTTP admission/upload/credential routes, durable restart, and architecture receipt emission. |
| Observable result | Both child digests produce immutable architecture-labelled S1-S3 evidence; native and emulated execution remain distinct. |
| Negative control | Missing child digest, unavailable architecture, missing observable, or failed production operation withholds v1.1.1 acceptance. |
| Isolation | Per-architecture scratch roots, databases, credentials, ports, logs, and cleanup owners; no v1.1.0 artifact or live deployment. |
| Prerequisites | Completed phase, passed local candidate receipt, immutable publication, and both child digests. |
| Final local gate | Later release work invokes the runner with published-artifact manifests at `evidence/gates/arm64.json` and `evidence/gates/amd64.json`. |
| Evidence | Future receipts under the arm64/amd64 manifests’ separate `evidence_root` directories with digest, execution mode, source/fixture identity, observations, exit code, and receipt hashes. |
| Limitations | Does not move v1.1.0, convert emulation into native proof, implement or execute the specified L3 runner, or establish customer-cluster recovery. |

## Execution boundary

The proposed runner and manifests are future requirements. L3 is specified in
tasks.md but remains unimplemented and unexecuted. `local.json`, `arm64.json`,
and `amd64.json` are future input manifests; receipts are separate outputs under
each manifest’s `evidence_root`, as defined in verification.md. Existing recovery
proof scenarios are enumerated in that contract’s `recovery` group.
Acceptance starts only after the lead
records every production implementation, fixture, configuration, and
documentation item as complete for this phase.
