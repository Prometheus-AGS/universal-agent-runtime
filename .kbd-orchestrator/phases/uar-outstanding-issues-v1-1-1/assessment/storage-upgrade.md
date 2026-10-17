# #385 storage and ownership upgrade assessment

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Assess. Written 2026-10-15.

Scope: persistence durability, authenticated knowledge visibility, and team execution ownership. Model selection and deployment binding diagnosis belong to the companion assessment. This document records findings and choices for analysis; it does not authorize implementation, deployment changes, data migration, or verification.

## Outcome and uncomfortable constraint

The reported upload failure follows a specific production guard: the new durable ingestion path refuses remote SurrealDB unless the trusted host explicitly attests durability. The deployment evidence supplied by the lead is consistent with that guard being unset. A working HTTP database connection and a single replica do not establish durable storage. Enabling the setting without establishing the server's actual backing would make an unsupported durability claim.

The reported empty replacement knowledge base has a separate explanation supported by source: authenticated knowledge moved from subject-only ownership in legacy tables to complete verified-principal ownership in disjoint tables. Existing documentation deliberately preserves old data without automatically adopting it. No live database inspection has established where this deployment's old records are, their counts, or whether the replacement base was created by a consumer.

The team ownership warning is also separate. It reports a retained execution claim that this process does not own. The mechanism predates the upgrade and is not by itself proof that ordinary chat must fail. A dead predecessor without qualified exclusion evidence may remain blocked; neither timeout nor replica count is acceptable evidence for automatic takeover.

## Evidence classification

### Operator and lead observations

- #385 reports that `3e6af7c4` worked while `76f2ad30` / v1.1.0 fails chats, shows a different empty KB, rejects uploads with HTTP 503 durable storage unavailable, logs the disabled durable ingestion dispatcher, and logs `TEAM_EXECUTION_OWNER_CONFLICT` at startup.
- The deployment uses remote SurrealDB at `http://surrealdb:8000`, JWT verification through JWKS, and one reported runtime replica.
- The lead's read-only deployment inspection reports that the know-me Pod currently runs v1.1.0 and that deployment/config-map configuration plus secret key names do not supply `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED`.
- The lead observed dispatcher-disabled and team-owner-conflict startup messages. This assessment did not independently repeat production reads or inspect secret values.

### Source and history observations

The investigation used source reads, `rg`, `git show`, and `git diff` between the reported baseline and new revision. No tests, builds, compiler checks, formatting, linting, packaging checks, or review gates ran. The inspected baseline already contains both the remote durability flag and nonexpiring team execution ownership. The upgrade adds durable upload dependence on that flag and adds supervisor-backed exclusion to the existing reclaim contract.

### Not established

- Actual live SurrealDB engine, volume durability, server version, backup/recovery behavior, and cold restart survival.
- Live source and destination namespace/database identities or legacy/new KB row contents.
- Whether JWT issuer, tenant, or subject changed between the old and new deployment.
- Which predecessor owns the team claim, whether it or its descendants survive, whether shutdown completed, or whether qualified fencing evidence exists.
- Whether the reported ordinary chat failure shares a cause with either storage warning.

## 1. Durable uploads

### Implemented contract

`SurrealDbProvider::new_with_auth` in `src/uar/persistence/providers/surreal.rs` authenticates, selects namespace/database, and initializes storage. It then captures durability once at construction:

- Local `surrealkv://` is eligible.
- A remote server endpoint is eligible only when `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` is exactly `1`.
- `supports_durable_ingestion` returns that captured eligibility.

`upload_document` in `src/uar/api/knowledge.rs` checks this capability before accepting bytes and returns 503 with `Durable ingestion storage is unavailable` when false. `IngestionWorkerPool::start_durable_dispatch` in `src/uar/rag/ingestion_worker.rs` logs `Durable ingestion dispatcher disabled: storage does not support durable uploads` under the same condition.

The changed upload contract commits the document, immutable source bytes, and ingestion job before HTTP 202. The old path saved metadata and submitted bytes to an in-memory queue. The guard therefore protects a real durable acceptance promise, rather than an arbitrary backend allowlist.

`src/uar/persistence/providers/ingestion_records/surreal.rs` checks the same capability at storage entry points and implements transactional acceptance/publication. The dispatcher discovers committed jobs on startup and periodically; queue notification is an acceleration, not ownership of the upload.

### Assessment

Missing attestation is the strongest source-supported explanation for the observed upload 503 and dispatcher warning. The lead's deployment observations support this explanation. The remote backing has not been proven durable, so this is not authorization to set the flag now.

### Required #385 outcome

- Identify the actual storage eligibility reason and make the remediation legible to operators.
- Restore upload operation through a supported durable configuration once its backing is established.
- Retain rejection when the configured store is unsupported or unattested; never acknowledge ephemeral upload acceptance as durable.
- Explain the required setting and construction-time capture in upgrade/setup guidance.

### Candidate implementation choices for Analyze

Prefer a consistent structured reason such as the already-used `remote_surreal_durability_not_attested` across diagnostics and relevant capability/error responses. Determine the existing client error contract before changing an upload response shape. A new typed config field is optional scope, not required merely because the environment setting exists. If selected, preserve the existing environment compatibility and never infer attestation from an HTTP URL or a successful health response.

### Sources

- `src/uar/persistence/providers/surreal.rs`: `new_with_auth`, `supports_durable_ingestion`, `channel_observer_unavailable_reason`.
- `src/uar/api/knowledge.rs`: `upload_document`.
- `src/uar/rag/ingestion_worker.rs`: `start_durable_dispatch`.
- `src/uar/persistence/providers/ingestion_records/surreal.rs`: `require_durable`, `accept`, `publish`.
- `docs/operations/durable-ingestion.md`.
- `.prometheus/gotchas.md`: remote durability qualification notes dated 2026-10-12.

## 2. Authenticated KB visibility

### Implemented contract

`UserContext::session_owner_key` delegates authenticated ownership to `CredentialPrincipal::storage_key`. The key includes issuer kind, issuer, optional tenant, and subject with explicit framing. An actual anonymous context retains its separate compatibility path.

`src/uar/api/knowledge.rs` changed authenticated list/read/write operations from `user.user_id` to this complete owner key. `KnowledgeNamespace` routes authenticated records to `principal_knowledge_bases`, `principal_knowledge_documents`, `principal_knowledge_chunks`, and `principal_knowledge_ingestion`. A missing namespace marker means legacy, including a subject string that happens to resemble a new principal key.

`migrations/surrealdb/principal_knowledge.surql` creates independent tables and indexes and explicitly performs no adoption or rename. New authenticated ingestion jobs use `principal_v1`; legacy nonanonymous jobs remain preserved and undispatched. Runtime history has an analogous deliberate isolation change, described in `docs/operations/session-principal-isolation.md`.

### Assessment

The new visibility boundary can explain why the same signed-in user sees an empty authenticated KB collection after upgrade even when old data remains. It does not prove that this particular deployment's records survive, that it uses the same namespace/database, or that its JWT identity is unchanged. The creation of a differently identified empty base must be traced separately if its creator matters to #385 resolution.

### Required #385 outcome

- Explain and diagnose the old/new owner and storage namespace transition without exposing another principal's records.
- Determine whether the reported source data remains and whether identity/configuration changed using separately authorized evidence collection.
- Provide an explicit supported recovery path. Current documentation permits authenticated creation/reupload while preserving legacy data.
- Keep full-principal isolation and reject automatic adoption based only on matching subject text.

### Optional candidate: privileged legacy KB import

A new legacy KB migration feature is not implicitly mandatory for #385. Analyze must decide whether supported reupload satisfies the issue's recovery need or whether preserving existing KB IDs/chunks requires separately scoped import work. No existing KB import API was identified; documentation explicitly describes migration as future work.

If import is selected, reuse the authority pattern already implemented for memory import in `src/uar/api/memory_admin.rs` and `src/uar/memory/migration.rs`: trusted-host administrator, separately verified target owner credential, and a retained mapping-evidence reference. Subject equality alone is not mapping authority. The evidence reference is an auditable operator decision, not a newly invented authorization token.

The prospective import contract must specify preview, selected source identity, source snapshot digest, a durable unique source-to-principal mapping receipt, idempotent replay, and rejection of competing mappings or changed source. Copy the selected KB and its complete document/chunk/job closure while preserving originals. Preserve logical IDs, configuration, and embedding identity; explicitly rewrite owner keys and physical references. Reject destination ID/name collisions rather than silently merge. Require quiescent legacy writers and compare the source snapshot at commit. Publish an entire bounded bundle atomically or explicitly design a staged publication protocol before implementation; do not expose a partial imported base.

Legacy pending/processing documents without retained source bytes must remain failed with reupload required. Existing indexed content may be retained only with coherent chunks and embedding provenance. An import must not invent source bytes, claim indexing completion, or make nonanonymous legacy jobs dispatchable merely by editing their marker.

### Sources

- `src/uar/security/claims.rs`: `session_owner_key`, `CredentialPrincipal::from_verified_context`, `storage_key`.
- `src/uar/persistence/knowledge_namespace.rs`.
- `migrations/surrealdb/principal_knowledge.surql`.
- `src/uar/persistence/providers/ingestion_records/surreal.rs`: initialization and namespace-sensitive discovery.
- `docs/operations/knowledge-principal-isolation.md`.
- `docs/operations/session-principal-isolation.md`.
- Reusable authority/idempotence precedent: `src/uar/api/memory_admin.rs::import_legacy_memory_handler`, `src/uar/memory/migration.rs::import_legacy_memory`.

## 3. Team execution ownership

### Implemented contract

Each `CollaborationCatalogService` receives a fresh incarnation ID. `acquire_execution_owner` accepts an existing matching held claim or advances a released claim; a held/draining predecessor produces `TEAM_EXECUTION_OWNER_CONFLICT`. This contract is already present in the reported working baseline.

`src/server.rs` catches startup acquisition failure, logs that the catalog is read-only for the executor, and continues startup. Collaboration capability projection disables team/workflow execution when this process does not own the catalog. Ordinary chat causality cannot be inferred from this warning alone.

Existing collaboration routes expose `/execution-owner`, `/execution-owner/quiesce`, and `/execution-owner/reclaim`. Privileged quiesce/reclaim requires the existing authenticated owner and admin-key check. Quiesce joins owned team work and records trusted evidence. Orderly server shutdown releases only after cleanup joins have succeeded.

Abrupt recovery uses the separate trusted supervisor contract. A registered predecessor can be challenged and excluded through `/api/uar/recovery/supervisor/challenges` and `/{id}/exclude`; a sealed proof is staged for reclaim. These supervisor APIs require verified installation/service host authority, not an ordinary admin role or caller-supplied proof JSON. Reclaim uses expected epoch and exact fence, consumes evidence once, advances incarnation/epoch, transfers queued attempts, and leaves running/cancelling effects uncertain with reserved accounting.

### Required #385 outcome

- Report which ownership state blocks team execution and distinguish it from database and ordinary-chat errors.
- Exercise the clean shutdown/restart handoff after full phase implementation.
- Document and expose existing supported recovery paths with the required authority and evidence.
- Preserve nonexpiring fences, uncertainty, and accounting liabilities. Do not add a single-replica exemption, timeout expiry, claim deletion, or force-reclaim flag.

### Choices and limits for Analyze

Determine whether #385 has an actual clean shutdown defect or an abrupt predecessor without evidence before changing lifecycle code. Existing source already implements release after successful joined cleanup; a log warning is insufficient grounds for replacing it.

If the prior process remains reachable, use its existing quiesce path and evidence-backed replacement reclaim. If it is dead and has prior qualified supervisor registration, use the existing exclusion path. If it is dead and unregistered, no safe generic takeover mechanism was identified. A new infrastructure fencing adapter would be new scope and needs its own authority/containment design; do not hide it inside a patch described as startup cleanup.

The shipped Kubernetes exclusion profile requires independently supervised qualified CRI/cgroup infrastructure and can inhibit scheduling on an entire node. It is not equivalent to checking Pod absence or restarting a Deployment. This assessment authorizes none of those operations.

### Sources

- `src/uar/compiler/collaboration/service.rs`: constructor and `with_service_instance`.
- `src/uar/compiler/collaboration/team_execution/ownership.rs`: acquire, joined evidence, release, supervisor evidence, reclaim.
- `src/uar/runtime/team_execution/controller.rs`: `quiesce`, `shutdown`.
- `src/server.rs`: startup claim acquisition and joined-cleanup release.
- `src/uar/api/collaboration/team_execution.rs`: ownership/quiesce/reclaim routes and privilege check.
- `src/uar/api/supervisor.rs`.
- `src/uar/domain/team_execution.rs`: fence, reclaim request, evidence and receipt contracts.
- `docs/operations/supervisor-recovery.md`.
- `docs/agents/collaboration/team-execution-profile-migration.md`.

## Proposed acceptance after the full implementation boundary

These are scenario requirements for planning, not commands executed or permission to verify early.

1. Real remote SurrealDB without attestation rejects upload with an actionable reason and never returns 202; a qualified durable profile accepts and recovers committed bytes/jobs through joined process restart.
2. Startup and capability diagnostics accurately distinguish unattested storage, unavailable storage, and nonowned team execution without exposing credentials.
3. Upgrade from baseline-shaped legacy KB data preserves original records, isolates authenticated principal data, and supports the chosen recovery path. Equal subjects under different issuers/tenants remain isolated.
4. If optional import is selected: actual production API/backend exercise of preview, atomic publication, unchanged replay, changed-source refusal, competing-owner refusal, destination collision, missing source bytes, embedding provenance, cross-owner dependent-row refusal, and cold readback.
5. Team clean shutdown/restart releases and reacquires safely; a second executor cannot acquire a held fence. Qualified recovery consumes exact evidence once, advances epoch, transfers only queued work, and retains uncertain effects/accounting. Missing proof remains blocked.
6. Correlate the separately diagnosed chat path with the repaired storage/owner configuration before declaring #385 complete; passing uploads alone does not close the issue.

## Assessment completion record

Changed only this assessment document. No production source, configuration, data, dependencies, or guards were changed. No verification command or gate ran, in accordance with the full-phase boundary. Runtime behavior, live database contents, durable backing, and deployment recovery remain unverified for the reasons above. Optional legacy KB import and any new configuration surface remain decisions for Analyze rather than accepted implementation scope.
