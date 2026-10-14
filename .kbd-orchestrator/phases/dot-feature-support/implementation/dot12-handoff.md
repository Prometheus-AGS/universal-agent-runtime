# Dot12 implementation handoff

Phase: parent Dot Execute. Work is source-authored, uncompiled and unverified. No KBD task status was changed. Full expanded Dot implementation inventory is still required before any gate.

## Files changed

- `src/uar/memory/owned.rs` (new): complete CredentialPrincipal adapter for add/search/list/hybrid/direct-ID/update/delete/history/compression/capture/TaskStreams. Mandatory owner predicates; protected metadata/record identity; source-first ownership denial. History carries imported legacy history after proving current owner.
- `src/uar/memory/service.rs`: binds ordinary calls to the adapter, disjoint `__owner_v1` database, retains embedding service for migration, gates raw admin/graph/mindmap access on trusted host, requires owner context for TaskStreams, propagates background expiry failures.
- `src/uar/memory/migration.rs` (new, surreal-backend): deterministic evidence-backed import through independent session on existing engine, re-embedding, protected provenance/history, replay/conflict checks and untouched quarantine source.
- `src/uar/memory/mcp_server.rs`: owner-bound memory/TaskStream dispatch, trusted-host-only shared graph, verified HTTP request Parts extraction, session owner binding for POST/GET/DELETE.
- `src/uar/memory/scopes.rs`: removes anonymous downgrade and unrestricted query fallback; classification no longer grants access.
- `src/uar/memory/context_builder.rs`: JSON-escaped recalled records explicitly labeled untrusted data without authority.
- `src/uar/memory/mod.rs`: exports owned adapter and feature-gated migration module; scoping documentation.
- `src/uar/api/memory.rs`: legacy endpoints route through verified owner adapter and reject mismatching payload owner filters.
- `src/uar/api/memory_admin.rs`: removes forged admin contexts derived from query filters, requires actual trusted host, adds operational `/import-legacy` endpoint with separately verified optional target owner JWT.
- `src/uar/admin/memory.rs`: scoped ordinary SDK direct-ID operations, explicit trusted host administrative branch, shared creation partition; adapts existing unavailable-memory fixture signatures only.
- `src/uar/tools/memory.rs`: context-free calls fail, native context's existing ActorOwner::credential_principal supplies authority; actual RecordId wire conversion supplies usable direct IDs.
- `tests/dot_memory_owner_contract.rs` (new): authored real-store/native/MCP/local/shared scenarios; no execution.
- `openspec/changes/dot-12-memory-isolation/operator-memory.md` (new): exact migration operation, namespace/compatibility contract, deferred gate and limitations.

## Root integration contract

Root already reported adapting `sdks/rust/src/runtime.rs` direct-ID get/update/delete to pass actual UserContext. No server.rs or manager.rs edits made. Root inspected their background injection/capture paths and confirmed original verified UserContext is retained. Native ActorOwner already carries complete principal; reconstructed legacy actors lacking it fail closed.

Service get/update/delete/history now take final `&UserContext`. Public TaskStream methods replace optional user ID with verified UserContext; create adds final UserContext. Public graph/mindmap methods add final UserContext and require trusted host. Source search found no external in-repo graph/mindmap service callers requiring adaptation. UarMemoryMcpServer::new now takes Arc<MemoryService>, not raw storage. `storage()` is crate-internal; external fixtures must use scoped service methods or verified admin_storage.

The existing auth middleware must precede the memory MCP router so request extensions contain the actual UserContext. The router enforces complete principal on all HTTP requests and binds returned session IDs to that principal. Shared knowledge-graph operations have explicit admin enforcement; no ordinary cross-tenant fallback exists.

## Decisions and uncomfortable limits

A separate owner database prevents a legacy literal subject resembling the encoded owner key from being accepted as a new scoped row. This quarantines all unsuffixed memory-domain records rather than guessing identities; explicit import handles memory records and preserves unattached TaskStream provenance. Shared legacy graph/TaskStream/mindmap migration is not automatically invented. Context-free direct SDK/tool use that previously relied on subject-only ownership must provide a verified host/user context.

Migration endpoint requires trusted host + optional separately verified owner issuer JWT. The existing identity-only verifier rejects API-key JWTs; current caller can omit secondary JWT. Mapping evidence is operator responsibility and retained in protected metadata, not treated as credential proof. Legacy writers must be quiesced. No automatic owner reassignment. No edits to vendored code, persistence providers, migrations directory, dependency pins, server or manager.

Guards trace to the explicit Dot12 owner boundary: required owner, immutable owner/record identity, payload filter rejection, protected provenance, foreign direct-ID denial, session owner binding, trusted admin check and migration conflict check. Compression/expiry avoid the pinned helper's observed source-level suppression of durable deletion failures. No new retries/timeouts.

## Remaining unverified

No tests, compilation, lint, formatting, Cargo commands, external gates or review gates ran. Only file/source inspection and implementation writes occurred. Authored tests are not acceptance. Full runtime process restart, SDK/HTTP admin composition, import endpoint credential handling, all supported backend profiles and customer build remain phase-end obligations. Deferred command and required environment are recorded in operator-memory.md. Do not mark task12 complete from this handoff.
