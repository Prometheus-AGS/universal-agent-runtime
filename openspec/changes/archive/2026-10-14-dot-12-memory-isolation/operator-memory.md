# Complete-owner memory operation

Status: production source and acceptance scenarios authored; no compilation or executable acceptance has run. Full expanded Dot implementation remains the verification boundary.

MemoryService continues to use the pinned surreal-memory engine and its embedding/index implementation. The effective memory database is `<memory.database>__owner_v1` within the configured namespace and existing embedded path or remote endpoint. The unsuffixed database is quarantine. Do not configure `memory.database` to the suffixed name: that would create a second suffix and change the quarantine source.

Every ordinary HTTP, native, MCP, background capture, search and context-retrieval operation requires the complete verified CredentialPrincipal. The key encodes issuer kind, issuer, tenant and subject. `user_id`, `agent_id`, `session_id`, scope and task names are query dimensions, never authority. An optional request `user_id` must equal the authenticated subject; it cannot specify the encoded key or another owner. All ordinary scopes, including `global` and `agent`, stay inside that owner partition. Anonymous and synthetic UserContext values fail closed.

Legacy `POST /memory` and `GET /memory` remain available, with verified owner capture replacing subject-only storage access. Native tools require `call_with_context` with an ActorOwner retaining its CredentialPrincipal; context-free calls fail. MCP HTTP tools use verified UserContext in the current HTTP request. An MCP session ID is bound to its initializing principal for POST, SSE GET and DELETE; another principal receives 403. Shared graph tools require explicit trusted-host authority because the existing graph schema has no tenant ownership column. TaskStreams bind their memory, deduplication, context and summary filters to the same complete owner.

`MemoryService::{get,update,delete,history}` now require `&UserContext`. `for_owner(&CredentialPrincipal)` provides the equivalent host-bound adapter for native execution. The public service's TaskStream methods accept verified context instead of an optional owner string; graph and mindmap methods require explicit trusted host context. Raw storage is crate-internal except for `admin_storage(&UserContext)`, which requires `trusted_host_id()`. SDK administration get/update/delete preserve ordinary own-memory access, while trusted hosts retain explicit cross-owner administrative access.

The existing `/api/admin/memories` collection is a trusted-host administration surface. A signed role label alone does not grant installation authority. Its user filter remains a literal filter, never a synthetic caller. Shared creation uses the reserved `uar:shared:v1` partition and does not deduplicate against another owner's records. Ordinary callers cannot read that partition. Per-owner creation uses the ordinary authenticated endpoint.

## Legacy import procedure

The uncomfortable thing: quarantine makes old memories unavailable until their owner mapping is proven; it does not reconstruct missing issuer or tenant history. Subject-only records cannot be automatically assigned merely because the current caller has the same subject. Old strings resembling a new `v1:k:...` key remain literal legacy subjects.

1. Stop every writer that still uses the unsuffixed database, and retain its backup. The new runtime writes only to the suffixed database.
2. Establish the exact legacy record-to-owner mapping from operator evidence. If issuer or tenant is ambiguous, leave the row quarantined. Do not bulk infer ownership from subject strings.
3. Authenticate the administrative request through the existing trusted host path. For another owner, obtain that owner's valid issuer JWT through the normal identity provider. API-key JWTs are rejected by the identity-only secondary verifier, which cannot check durable API-key revocation. No bearer is stored in the receipt.
4. Submit `POST /api/admin/memories/import-legacy` with JSON:

   ```json
   {"id":"memory:legacy-record-id","evidence":"operator-change-record/reference","owner_bearer":"verified-owner-JWT"}
   ```

   Omit `owner_bearer` only to map to the actual verified administrative caller. There is no `owner_id`, `tenant`, or `issuer` assertion field. The legacy row's literal subject must match the verified target's subject, and the trusted operator remains responsible for the mapping evidence.
5. Retain the returned memory ID and its `metadata.uar_migration` receipt (available through trusted `admin_storage` or the owned memory/MCP read). The receipt records source namespace/database/ID, source snapshot digest, operator evidence reference, trusted host ID, original metadata/time/task link and history. The destination is deterministically derived from the source identity. Repeating an unchanged import returns that destination. Conflicting owner assignments or changed legacy source snapshots return a conflict without overwriting the earlier mapping.

The service clones the existing SurrealDB session before selecting the legacy database, so importing local SurrealKV does not open the same filesystem store twice. It re-embeds the imported content using the active provider and commits the destination plus its creation history through `store_indexed_memory`. Preserved legacy history remains in the protected migration envelope and is included in owner history retrieval. Task links are retained as provenance but are not attached to an unproven legacy TaskStream. Legacy originals are never deleted or rewritten by this operation.

Trusted in-process hosts can invoke the same operation as `service.import_legacy_memory(&administrator_context, legacy_id, &verified_owner_context, evidence).await`. Both contexts must have been captured by authentication; constructing subject fields does not establish authority.

Rollback must keep the quarantined database inaccessible to ordinary callers. Reverting to a subject-only server against the unsuffixed database would reopen the original isolation defect and is not a safe rollback.

## Pending production acceptance

Authored target: `tests/dot_memory_owner_contract.rs`. Deferred command after the lead records the full-phase implementation inventory:

```sh
cargo test --locked --no-default-features --features server-full --test dot_memory_owner_contract -- --test-threads=1 --nocapture
```

The shared case requires `UAR_DOT_MEMORY_SURREAL_URL` and optional `UAR_DOT_MEMORY_SURREAL_USER` / `UAR_DOT_MEMORY_SURREAL_PASS` pointing to explicitly isolated durable infrastructure. Both cases use the configured production local embedding provider and may require its model files. The source exercises local SurrealKV, two service instances on shared SurrealDB, native execution, the actual MCP HTTP transport, equal subjects across issuer/tenant, direct-ID denial, context capture, quarantine and migration conflicts. These are unexecuted scenarios, not passing receipts. Cold-process restart, full runtime HTTP/SDK composition, migration HTTP credential handling and the required customer build remain final phase gate responsibilities.

MCP API grounding: the installed rmcp 3.4.1 `StreamableHttpService` source and official [request extension documentation](https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/tower/struct.StreamableHttpService.html) show HTTP request Parts in RequestContext extensions. SurrealDB session cloning was inspected in the installed pinned 3.3.0 SDK source; no dependency upgrades were made.
