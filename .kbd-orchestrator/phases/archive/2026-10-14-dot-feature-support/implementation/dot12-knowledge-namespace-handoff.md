# Dot12 task3 knowledge namespace source handoff

Status: production source authored; all verification deferred. Full expanded Dot phase boundary remains closed. No tests, checks, formatters, builds, probes, validators, product reviews, deployments or cluster operations ran.

## Contract and decision

A2A owns caller APIs, manager, sessions and policies. Its `UserContext::session_owner_key()` and `ActorOwner::session_owner_key()` return the verified full principal key or explicit anonymous boundary. Existing knowledge method signatures and API model constructors remain unchanged. Adapter owner inputs other than `v1:k:*` or literal `anonymous` now fail. The prefix is a storage-routing discriminator for trusted host callers, not an authentication mechanism. No API may accept a caller-selected owner string in place of verified context.

Authenticated tables/maps are disjoint from every old subject. No matching legacy owner string is adopted. An old ingestion job with a canonical-looking owner remains Legacy through its omitted marker and cannot be read as authenticated source, transitioned, prepared or published. Background discovery excludes legacy nonanonymous jobs and verifies source-table namespace. API job reads verify persisted owner/document/namespace identity. Anonymous legacy jobs preserve their prior serialization for CAS.

The uncomfortable upgrade behavior: old authenticated KBs remain preserved but unavailable until a separately authorized migration with independent ownership evidence exists; users can reupload to freshly owned bases. No migration tool, automatic adoption or recovery authority was invented.

## Files changed

- `src/uar/persistence/knowledge_namespace.rs`: new namespace selector and fixed host-query/table mapping.
- `src/uar/persistence/mod.rs`: module export and knowledge caller contract comment only.
- `src/uar/persistence/providers/{postgres,surreal,memory}.rs`: knowledge-only CRUD/search/count/status/delete namespace selection; separate memory maps; existing knowledge fixture owners adjusted. Session/policy teammate spans preserved.
- `src/uar/persistence/ingestion_records/mod.rs`: durable namespace marker, fallible new-job creation, immutable namespace transition/publication validation.
- `src/uar/persistence/providers/ingestion_records/{postgres,surreal}.rs`: namespace-specific acceptance/source/job/CAS/publication/delete, dual discovery with legacy anonymous restriction, namespace-scoped reconciliation; Surreal schema startup registration.
- `src/uar/rag/{ingest,ingestion_worker}.rs`: provenance validation before preparation/dispatch only.
- `migrations/20261013010000_principal_knowledge.sql`: new PostgreSQL tables, composite keys/FKs, indexes; no legacy migration/adoption.
- `migrations/surrealdb/principal_knowledge.surql`: independent schema definitions; included from existing ingestion initialize called by shared host migration path.
- `tests/principal_knowledge_namespace.rs`: authored real adapter collision/source/publication/cold-process fixture, unexecuted.
- `tests/knowledge_base_integration.rs`: canonical fixture owners only, unexecuted.
- `docs/operations/knowledge-principal-isolation.md`: compatibility, migration and provenance contract.

A2A independently updates `tests/durable_ingestion.rs` to use the real verified JWT owner rather than a synthetic unverified UserContext. A2A confirms server snapshot/poll uses `list_knowledge_bases(full principal)` with no literal knowledge-table live subscription. No new generic PostgreSQL notification trigger was added; authenticated snapshot/poll remains the existing owner-aware projection path.

## Remaining / unverified

No local or shared backend behavior is certified. Compilation, migration execution, vector reads, CAS races, restart and HTTP coverage remain unverified under the operator's full-phase rule. New adapter fixtures require serialized real-path phase validation; the PostgreSQL scenario needs a dedicated empty pgvector database. No canonical KBD status or task checkbox was changed. Caller authentication correctness remains in A2A's packet; this adapter packet does not mint or verify credentials. No LKE or local service was launched.

Guards trace to the explicit observed collision requirement and host/persistence trust boundary: reject unqualified nonanonymous subjects; refuse legacy namespace reinterpretation; enforce source-table/job identity before use. No unrelated safeguards or speculative features were added.

## Source composition addendum: shared SurrealDB (matrix 104)

The bounded follow-up changes only `tests/principal_knowledge_namespace.rs` and its operations document. The fixture now has an unignored network SurrealDB case requiring `UAR_DOT12_SURREAL_URL` (ws/wss/http/https only), optional paired `UAR_DOT12_SURREAL_USER` / `UAR_DOT12_SURREAL_PASS`, and production root signin. It uses a fresh generated namespace shared by three joined child processes, with the same real legacy collision, isolation, ingestion publication and cold-read assertions as KV and PostgreSQL. Missing network resources fail rather than skip. Credentials default to the adapter's existing root/root behavior when neither is supplied. No synthetic durability attestation is set. PostgreSQL still needs its existing ignored scenario included by the final whole-target gate and its dedicated pgvector database.

The network test establishes client-process persistence and namespace behavior when eventually executed; it does not establish server crash durability. The isolated remote namespace is retained for inspection. All fixture behavior, compilation and runtime interactions remain unverified. No commands invoking verification, services, cluster operations or deployments ran during this addendum. A2A session/history/erasure fixture changes were not touched. Files are released to the lead after source handoff.
