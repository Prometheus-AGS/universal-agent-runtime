# Provider credential authority and migration

Ordinary provider capture requires a host credential-source decision even when
`CREDENTIAL_ENCRYPTION_KEY` is absent. The selected source is bound to verified
identity, the resolved catalog provider, exact endpoint and captured policy.
MCP descriptions, annotations, role labels and decoded token hints confer no
credential authority. Kernels cannot install policies, mutate credentials or
retry through another client after a denial.

The uncomfortable case is a preview that looks ready while another replica rotates
the source. Preview is a metadata observation; migration compares its revision in
the actual copy transaction. Quiescing every writer remains necessary for a complete
cutover inventory. A lost response may have committed: retry the exact authorized
operation to discover its immutable receipt.

## Startup configuration

The trusted configuration object is `security.provider_credentials`:

```yaml
security:
  provider_credentials:
    mode: shared
    installation_id: 670b34d7-8080-4e03-bdaf-45a6386eb04b
    shared_fallbacks:
      - issuer: https://issuer.example
        tenant_id: approved-tenant
        provider_id: openai
        source: system
    principal_mapping_file: /operator/fixed-local-mappings.json
```

The UUID and identities above are illustrative, never recovery evidence. Provision
a deliberate canonical lowercase hyphenated UUID for each local installation.
Never generate a replacement during startup. A changed UUID is a new namespace;
there is no database-global singleton or automatic detection of accidental change.

`mode` is `single_owner` (omitted default) or `shared`. Remote single-owner is an
explicit supported deployment choice. Shared requires a verified issuer identity,
signed nonempty tenant and subject. Local launch, issuerless JWT, anonymous and
synthetic contexts cannot obtain shared credential authority. An issuer-qualified
verified API-key principal remains eligible.

Resolution is session, agent, user, authorized system, authorized config. Shared
house grants match the exact issuer, tenant, resolved catalog provider and source.
A system grant never grants config. Wildcards and duplicate tuples are invalid.
Denied system resolution makes no system-row lookup. Store or decrypt errors are
errors, not misses. Credential-free decisions prohibit ambient environment pickup.
Opaque supplied clients need host-verifiable provenance. Captured children reuse
or narrow their root decisions; a reload does not widen an admitted run.

Issuer-qualified owner keys are unchanged. Local credential authorities become
`urn:uar:installation:<UUID>:jwt` and `urn:uar:installation:<UUID>:host`, retaining
issuer kind and length-framed tenant/subject. General principal, session, schedule,
followup and knowledge owner bytes remain unchanged. Local credential access
without a UUID fails; no plausible old owner is returned. A bound destination
remains usable, while an unresolved old fixed-local source blocks ordinary missed
lookup or creation with `credential_migration_required`.

The existing encryption and `CREDENTIAL_OWNER_MAPPING_FILE` environment inputs
remain. Missing encryption disables stored credential operations, not policy.
Configured fixed-local mapping files are read and strictly validated at startup,
including when encrypted service is absent. Policies are immutable until existing
host reconfiguration installs a new instance.

## HTTP management

All paths below are relative to `/api/uar/credentials`.

| Method/path | Contract |
|---|---|
| `GET /principal` | Bound `owner_key`, credential `issuer`, `tenant_id`, `subject`. |
| `GET /`, `GET /{provider}` | Existing masked metadata only. |
| `PUT /{provider}` | Strict `{ "api_key": "value" }`; new row revision on every accepted write. |
| `DELETE /{provider}` | Delete the caller's row; no journal deletion. |
| `GET /migration-preview?limit=100&cursor=...` | Verified trusted-host authority only. Admin role/settings key alone is insufficient. |
| `POST /migrate-legacy` | Strict existing `{scope, legacy_scope_id, provider_id, reason}`. |
| `POST /migrate-principal` | Strict `{source_owner_key, scope, source_scope_id, provider_id, reason}`. |

The new principal reason is exactly
`operator_approved_local_namespace_migration`. The source revision, destination
and approval come only from operator configuration. Legacy reasons remain free
form for compatibility; keep them nonsecret. Never put secrets in receipts or
approval IDs.

New safe failures use `{"error":{"code":"..."}}`: identity ineligible is 403
`credential_identity_required`, missing local UUID is 503
`credential_installation_id_required`, unresolved old source is 409
`credential_migration_required`, denied house source is 403
`credential_source_denied`. Unauthenticated requests retain 401 and unavailable
encrypted service retains 503.

Migration mapping denial is 403 `credential_mapping_denied`; missing sources are
404 `legacy_credential_missing` / `principal_credential_missing`; destination or
journal conflicts are 409 `credential_mapping_conflict`; source/precondition drift
is 412 `credential_migration_precondition_failed`; persistence failure is 503
`credential_migration_unavailable`. Invalid principal request shape/reason is 400.

## Supported metadata upgrade and cutover

1. Back up both credential tables, the migration journal, encryption material,
   startup configuration and reviewed approvals as one matched recovery set.
2. Stop every credential writer and consumer across replicas, including provider
   seeding. Install the additive schema and compatible runtime on every writer.
   Historical rows decode nullable revisions; old writers ignoring revisions are
   unsupported after this upgrade.
3. Provision deliberate mode and local installation UUID. With all writers still
   stopped, run the local command against the configured backend:

   ```text
   universal-agent-runtime --config /operator/uar.yaml credentials initialize-revisions --writers-quiesced
   ```

   This opens only the credential backend, starts no provider or listener, and
   returns safe initialization counts. The flag acknowledges operator quiescence;
   it cannot prove other replicas are stopped. Missing revisions are initialized
   once with fresh nonsecret UUIDs, preserving ciphertext, hints and timestamps.
   No preview or ordinary startup backfills them.
4. Start the controlled management host. Obtain the authenticated destination's
   `/principal` response and independently establish source ownership. Preview
   sources before installing exact approved mappings. Ambiguity stays quarantined.
5. Install the files below; restart/reconfigure the host consistently. Repeat
   preview, and copy its strong `precondition` tag unchanged to `If-Match` when
   applying as the destination owner. The tag is consistency data, not approval.
6. Compare complete inventory and receipts. Perform the phase's authorized real
   cold-read, rotation, deletion and concurrency observations before traffic.
   Admit shared traffic only with the intended exact source grants.

The legacy mapping file selected by `CREDENTIAL_OWNER_MAPPING_FILE` contains a
strict array of `{scope, legacy_scope_id, provider_id, destination_owner_key,
approval_id}`. The new `principal_mapping_file` contains strict entries:

```json
[{
  "source_owner_key": "copy exact old fixed-local owner key from preview",
  "scope": "user",
  "source_scope_id": "verified-source-subject",
  "provider_id": "openai",
  "source_revision": "copy exact opaque source revision from preview",
  "destination_owner_key": "copy independently verified bound destination owner key",
  "approval_id": "nonsecret-independent-approval-reference"
}]
```

These placeholders intentionally are not valid installed mappings. Only old
`urn:uar:local:jwt` / `urn:uar:local:host` sources with their corresponding issuer
kind qualify. Destinations must be valid bound encodings. Scopes are user, session
or agent. User destination scope is its verified subject; session/agent scopes
retain the exact source scope ID and existing resource ownership requirements.
Reject duplicate sources, unknown fields and blank approval/source fields.

Each new/rotated/recreated credential receives a fresh UUID revision independent
of key contents and timestamps. No secret fingerprint is used. First apply copies
ciphertext/hint/original timestamps, assigns a fresh destination revision and
journals in one backend transaction. The source remains quarantined. Fixed-local
journal keys are `["principal-fixed-local-v1", owner, scope, scope_id, provider]`;
legacy three-element keys stay unchanged. Revisions never enter journal uniqueness.

Both routes accept one optional strong `If-Match`. Weak, wildcard, multiple or
malformed tags return 400 `credential_migration_precondition_invalid`; they never
become unconditioned operations. Legacy calls without the header preserve existing
semantics and are not preview-bound. An authorized replay returns the original
receipt after destination rotation/deletion or source disappearance. Changed
mapping/approval/fixed-local revision conflicts. Conditioned replay compares the
original receipt tag. A historical receipt without one cannot satisfy a conditioned
retry (412), but unconditioned historical replay remains valid. Never erase a
journal to bypass a conflict or resurrect a deleted destination.

## Preview and evidence limits

Preview accepts only `limit` (1–500, default 100) and an integrity-protected cursor
(maximum 4096 bytes), bound to host authority and policy/mapping revisions. Invalid
queries return 400 `credential_preview_request_invalid`; changed policy/mapping
returns 409 `credential_preview_stale`. Restart pagination then. Response fields
are `observed_at`, `policy_revision`, `mapping_revision`, `consistency` (always
`page_observation`), `items`, bounded-dimension `counts`, and `next_cursor`.

Items expose source/destination revisions, source metadata and installed approval
references, never ciphertext, hints, secrets, reasons or full receipts. Statuses
are `unmapped_quarantined`, `revision_unavailable`, `source_changed`, `ready`,
`missing_source`, `destination_conflict`, `journal_conflict`, `already_applied`,
`already_applied_destination_missing`, `already_applied_destination_changed`, and
`already_applied_revision_unknown`. Journal outcomes take precedence over readiness.
Inventory includes installed mappings whose sources are missing. Counts cover the
response's observation, not a frozen snapshot across pages; restart under quiescence
before cutover. Cancellation of preview performs no writes.

Rollback requires matched database/configuration/key snapshots under a compatible
binary, with explicit handling of subsequent rotations and deletions. Lost original
secrets and unrecorded rotations cannot be reconstructed. Subject-only or fixed
local-global shared access is not a supported rollback mode.

The phase's installed-image credentials group exercises local SurrealKV, shared
SurrealDB and PostgreSQL separately. Source fixtures supplement memory/SDK seams;
they cannot substitute for durable restart or provider account observations.
Consult actual phase receipts for command results. This document is not acceptance
evidence, and fixtures cannot establish real operator ownership or recovery proof.

Credential fixtures that sign JWTs inside the same process must first initialize
UAR's guarded verifier through the shared `initialize_jwt_verifier` fixture helper.
It submits a malformed token and requires 401 before any fixture token is signed;
it does not create authenticated identity or relax provider ownership. This order
also applies when the memory HTTP scenario runs alone, without a runtime fixture
booting first. The retained local-02 memory failure returned 500 during initial
owner authentication, before HTTP migration or storage assertions ran. Its repair
therefore requires a later authorized full-cohort gate; that earlier failure proves
no migration, atomicity or tenant-isolation result.
