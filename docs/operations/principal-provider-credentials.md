# Principal-scoped provider credentials

Provider credentials belong to the host-verified issuer, tenant and subject together. The HTTP API captures this identity from authentication; request bodies cannot choose an owner. Equal subjects in different tenants or issuers have different credential owners. Local JWT and authenticated local-host identities use explicit, distinct issuer namespaces.

Principal credentials use `provider_principal_credentials`, separate from the legacy `provider_credentials` table. A legacy subject that happens to equal a versioned owner key cannot access that owner's new records. Resolution retains session, agent, user, then operator system precedence. Each non-system lookup includes the captured principal. Actor and MCP binding cache ownership includes that principal. The credential resolver reads durable storage without retaining a secret cache.

The uncomfortable thing: legacy user, session and agent credentials stop resolving until an operator proves their ownership and installs an explicit mapping. Matching subjects alone are insufficient. Old observer records without verified principal proof cannot recover those secrets; they need an authorized host reconstruction. Do not restore a subject-only lookup as a rollback shortcut.

## Configuration and API

`CREDENTIAL_ENCRYPTION_KEY` enables the credential service. SurrealDB and PostgreSQL use their durable credential stores; explicit memory development mode remains ephemeral. An invalid encryption key or owner-mapping configuration fails server startup. An absent encryption key preserves the existing single-tenant configuration path.

All credential operations require an authenticated, consistent principal. Synthetic or anonymous user contexts cannot create one. The routes are mounted under `/api/uar/credentials`:

| Method and relative path | Behavior |
|---|---|
| `GET /principal` | Return the caller's `owner_key`, `issuer`, `tenant_id` and `subject` for an operator to verify. |
| `GET /` | List the caller's user-scope credentials as masked metadata. |
| `GET /{provider}` | Read masked metadata for the caller's provider; missing credentials return 404. |
| `PUT /{provider}` | Store or rotate the caller's key with body `{"api_key":"sk-example-1234"}`. |
| `DELETE /{provider}` | Delete the caller's credential; return 204, or 404 when absent. |
| `POST /migrate-legacy` | Apply an installed, operator-approved mapping to the authenticated destination owner. |

Read responses contain provider ID, last-four key hint and timestamps. They do not return plaintext or ciphertext. Storage and decryption failures are errors; a resolver error does not fall through to another account's environment credentials. An ordinary miss retains the configured operator system/environment fallback.

## Explicit legacy migration

1. Authenticate the destination through its intended production issuer and tenant. Read `GET /api/uar/credentials/principal` and verify the complete identity against independent operator records.
2. Identify the exact legacy scope, scope ID and provider. Prove which complete owner the source belongs to. Leave ambiguous records quarantined; do not infer ownership from subject equality.
3. Install an operator-controlled JSON mapping file and set `CREDENTIAL_OWNER_MAPPING_FILE` in the trusted host environment. The file is a JSON array. Each entry has this complete shape:

```json
[
  {
    "scope": "user",
    "legacy_scope_id": "legacy-alice",
    "provider_id": "openai",
    "destination_owner_key": "copy the exact owner_key from the verified principal response",
    "approval_id": "operator-review-2026-10-11-alice-openai"
  }
]
```

The destination string above is instructional sample data; replace it with the exact verified response. Valid mapped scopes are `user`, `session` and `agent`. System credentials retain their separate operator path. Unknown fields, empty approvals, empty destinations, system mappings and duplicate source mappings are rejected during configuration. Each runtime loads the mapping at startup; deploy the approved file consistently before accepting migration requests on multiple runtimes.

4. As the authenticated destination owner, submit `POST /api/uar/credentials/migrate-legacy` with:

```json
{
  "scope": "user",
  "legacy_scope_id": "legacy-alice",
  "provider_id": "openai",
  "reason": "Operator verified issuer, tenant and subject under the installed approval"
}
```

The request cannot supply a destination or approval. The service must find an exact installed source mapping whose destination equals the captured principal. Keep secrets out of the audit reason.

5. Retain the returned receipt with the operator approval. The destination ciphertext copy and migration journal commit together. User-scope migration changes the scope ID to the verified subject; session and agent scope IDs retain the mapped source value. Ciphertext and original timestamps are preserved. The source row remains in the legacy namespace, inaccessible to ordinary principal lookup.
6. Retry the same approved mapping after an interrupted response. An already committed mapping returns its original receipt. It does not overwrite a rotated destination or recreate a deleted destination. A conflicting destination, pre-existing destination credential, or changed approval for the same source is rejected. Do not erase the journal to retry a conflict.

| HTTP status | Error code | Operator action |
|---|---|---|
| 401 | Authentication required | Authenticate with a consistent host-verified identity. |
| 403 | `credential_mapping_denied` | Check the installed exact owner mapping and nonempty audit reason. |
| 404 | `legacy_credential_missing` | Check the source inventory; do not create a guessed ownership mapping. |
| 409 | `credential_mapping_conflict` | Resolve the conflict explicitly; preserve existing ciphertext and journal evidence. |
| 503 | `credential_migration_unavailable` | Restore storage availability, then retry the identical approved request. |

## Qualification scope

The integration target is `tests/credentials_api_integration_test.rs`. Credential qualification must exercise authenticated HTTP, real durable stores, resolver use, equal subjects across tenants and issuers, encoded-key collision attempts, rotation/deletion, restart, competing runtimes and migration replay/conflicts. A mock store or injected synthetic identity cannot establish these guarantees.

Local SurrealKV, shared networked SurrealDB and the modified PostgreSQL credential store require separate results. Credential-store acceptance does not certify PostgreSQL observer/checkpoint assistant parity. Consult the phase execution evidence for actual commands and outcomes; this procedure itself is not a passing test receipt.
