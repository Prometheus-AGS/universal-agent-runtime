# Credential authority contracts

Phase: `uar-outstanding-issues-v1-1-1`; stage: **Spec**; date: 2026-10-16.
Owner: `uar-trust-tools`. Authoritative behavior delta:
`openspec/changes/uar-outstanding-issues-v1-1-1/specs/tenant-isolation/spec.md`.
These are selected contracts for Plan, not implemented interfaces or acceptance
results. **Full-phase verification boundary CLOSED.**

## Scope, evidence and source-grounded reproducer

The uncomfortable case is a migration that appears ready in a preview but copies
a subsequently rotated secret, or a shared caller that reaches a supplied client
outside credential policy. Neither is prevented by merely masking HTTP output
or clearing `api_key`. The contracts below bind identity, source revision and
actual client capture. They preserve existing encrypted storage and avoid a
generic identity rewrite.

Read inputs: AGENTS.md, versions.toml, waypoint (revision 2930, Spec in progress),
team README/map/tool/skill policy, proposal, aggregate Analyze, analysis/trust.md,
decision-log.md, existing tenant-isolation specification, stored OpenSpec
spec-instructions-specs.json and the acceptance template. Applied skills:
agent-runtime-security, prometheus-rust-workspace and rust-best-practices from
their explicit paths. Security guidance supplies verified identity and host-only
authority; Rust guidance favors private capabilities, explicit fallible results
and narrow interfaces. No dependency or protocol changes are selected.

Compass's recorded graph refers to an older source commit; it is not current
runtime evidence. The following current-source recipes are **unexecuted** and
precede implementation:

| Recipe | Source and observed mechanism | Required contract |
| --- | --- | --- |
| Give two installations issuerless verified JWTs with the same subject/tenant and a common credential database. | `src/uar/security/claims.rs`, `CredentialPrincipal::from_verified_context`/`storage_key` derive the same fixed local authority. | Credential-only installation namespace; do not alter general principal bytes. |
| Miss every principal scope with a configured house key; repeat without encrypted service. | `src/uar/security/credentials/resolver.rs`, `resolve_with_context` ends in system resolution; `src/uar/runtime/manager.rs`, `apply_credential_layer` retains config when service is absent or all sources miss. | Independent policy handle applies even when service is absent. |
| Add only resolver refusal, then supply a primary driver or ordinary fallback. | `src/uar/runtime/turn/bindings.rs`, `RunModelBindings::capture` accepts supplied primary directly and constructs ordinary fallbacks from config. | Every capture must establish exact source authority before effects. |
| Change the local credential authority and attempt legacy migration of an old principal row. | `src/uar/security/credentials/principal_store.rs`, all `migrate_legacy` implementations read `provider_credentials` only. | New fixed-local principal source and disjoint journal identity. |
| Preview a source, rotate it with equal/reused timestamps, then apply its mapping. | `CredentialRecord` has timestamps but no revision; current migration checks exact tuple and destination, not a preview revision. | Nonsecret row revision plus atomic precondition, never a secret-derived fingerprint. |

The existing migration body and operator mapping use `deny_unknown_fields`.
Existing receipts return before recopy on replay and survive destination
rotation/deletion. The new design extends those boundaries rather than replacing
their protocol. No runtime exploit or successful migration has been observed in
this Spec task.

## T1 — policy and credential owner contract

All names in this section are **NEW additions**, except explicitly identified
existing names. The trusted startup object is `security.provider_credentials`:

| Field | Exact contract |
| --- | --- |
| `mode` | Enum `single_owner` / `shared`, default `single_owner`; explicit remote single-owner is valid. Unknown values reject config. |
| `installation_id` | Optional canonical lowercase hyphenated UUID string. When supplied, invalid syntax rejects config; absence fails only operations requiring a local credential owner. Never generate or infer it. |
| `shared_fallbacks` | Array, default empty; each entry has exactly `issuer`, `tenant_id`, `provider_id`, `source`. First three are nonempty exact strings, source is `system` / `config`. Reject wildcard fields and duplicate tuples; no role, subject-only or request-supplied grant. |
| `principal_mapping_file` | Optional trusted operator path to the separate fixed-local mapping JSON array described below. Unreadable/invalid configured file fails configuration safely. |

Keep the existing config/environment loader as the sole policy source; use its
existing nested configuration mechanism. Keep existing `CREDENTIAL_ENCRYPTION_KEY`
and `CREDENTIAL_OWNER_MAPPING_FILE` semantics. Missing encryption disables stored
credential service, not the policy handle. No new standalone environment policy
or live mutation endpoint is selected. Reload takes effect only through existing
host reconfiguration; already captured runs keep their admitted source subset.

Shared mode requires an identity already verified under configured issuer trust,
with nonempty signed tenant and subject; native launch, issuerless JWT, anonymous,
synthetic or inconsistent contexts cannot supply shared credential authority.
An exact issuer-qualified API-key identity captured through the existing verified
identity mechanism remains eligible. Headers, tool descriptions/annotations,
decoded unverified claims and role names do not confer authority.

Resolution order is session → agent → user → authorized system → authorized config.
Single-owner retains existing house compatibility for callers admitted by current
ingress rules. Shared requires an exact `(issuer, tenant_id, provider_id, source)`
grant for each house source after principal misses. A grant for issuer A cannot
authorize the same tenant name under issuer B. The provider is the resolved
catalog provider ID, including alias identity, not a display label or guessed
prefix. No system grant means no system-row lookup. Store/decrypt/identity errors
are errors, never misses; system miss cannot authorize config by itself.

A new host-created credential owner capability retains verified principal and
credential storage key separately. It has private construction and no request or
row deserialization. Principal store operations require this binding; raw general
principal keys cannot bypass it. Preserve the existing length-framed encoding and
issuer-kind distinction. For local JWT use authority
`urn:uar:installation:<installation_id>:jwt`; for local host use
`urn:uar:installation:<installation_id>:host`. Keep tenant and subject framing,
including the existing absent-tenant local marker. Issuer-qualified keys are
byte-for-byte unchanged. Captured delegations bind identically to their root.

Do not change general `CredentialPrincipal::storage_key`, session owners,
knowledge owners or schedule/followup grant encoding. No database-global
installation singleton is introduced. Multiple intentionally separate
installations can share one credential database. A valid changed UUID denotes a
new namespace; detection of accidental replacement is not promised. Memory store
identity and receipts last only for that process.

Existing `GET /api/uar/credentials/principal` keeps `owner_key`, `issuer`,
`tenant_id`, `subject`. It returns the bound credential owner and its credential
authority in `issuer`; only local authority changes. It returns no plausible old
owner when binding fails. Existing CRUD bodies and masked views stay unchanged.
New credential API errors use safe allowlisted codes: `credential_identity_required`
(403, verified identity ineligible for selected mode),
`credential_installation_id_required` (503, local identity not provisioned),
`credential_migration_required` (409), and `credential_source_denied` (403).
Unauthenticated requests retain 401; unavailable encrypted service retains 503.
Use `{"error":{"code":"<code>"}}` for these additions; never echo keys,
arbitrary provider URLs or backend messages. Existing legacy error codes remain.

## T2 — source integration handoff

The policy owner supplies a host-only decision/capability carrying bound owner,
resolved provider, selected endpoint/source and policy revision. Source classes
are principal session/agent/user, system, config, existing explicit request grant,
or explicitly credential-free. Only the credential-bearing object holds secrets;
metadata, logs and debug views do not. This is an internal contract, not a new
public serialized authorization token. The runtime writer selects Rust type names
in Plan with the trust owner and preserves existing SDK compatibility.

| Actual seam | Required integration before effects | Proposed Plan owner |
| --- | --- | --- |
| `src/config.rs`, `src/server.rs`, shared construction roots | Construct policy independently of optional encrypted service, validate config/mappings, pass it through HTTP and embedded host construction. | Lead-assigned single shared-root writer |
| `src/uar/security/credentials/{mod,resolver,principal_store,store}.rs` | Bind owners and gate each house source; return typed selected-source or denial; use metadata-only migration reads. | T1/T3 trust writer |
| `src/uar/api/credentials.rs` | Bind CRUD/principal destinations; require trusted host for preview; match exact migration mapping. | T3 explicit cross-domain assignment |
| `src/uar/runtime/manager.rs`, primary/team/graph capture | Resolve actual provider/source before client construction; reject missing shared identity before lookups. Preserve diagnostics, budgets and #384 timeouts. | T2 runtime writer |
| `src/uar/runtime/turn/bindings.rs`, `RunModelBindings::capture` | Admit primary, endpoint-profile and ordinary fallback sources, supplied clients, and captured inherited subset. A policy denial must not be swallowed as a fallback-constructor warning. | T2 runtime writer |
| Existing summarization/SDK/root/delegation capture callers | Consume the same authorized subset; unknown supplied clients require host-verifiable provenance or fail. Legacy SDK construction remains single-owner and cannot silently construct shared mode. | T2 runtime writer following source callers |
| `src/uar/api/providers.rs`, `src/uar/settings/manager.rs` | Preserve separate admin authorization and exact provider verification/seeding targets. Admin use does not grant run fallback. | Lead-assigned provider/settings writer |

Clearing config key is insufficient: constructors can resolve ambient keys. Deny
before building/accepting a client or sending a provider request. Credential-free
local inference needs a binding that prohibits ambient pickup. Existing explicit
request grants retain exact endpoint/provider checks and shared eligibility.
Descendants reuse/narrow root bindings; kernels never mutate stores, install
mappings, mint host authority or reconnect around a denial. NativeTool/McpRegistry
metadata is never authority to select provider credentials or migration scope.
This credential delta does not change MCP wire semantics or existing tool-policy,
approval, cancellation, redaction and exact-resource checks.

## T3 — additive migration and preview interfaces

**NEW fixed-local mapping format:** a strict JSON array of objects with exactly
`source_owner_key`, `scope`, `source_scope_id`, `provider_id`, `source_revision`,
`destination_owner_key`, `approval_id`. Strings must be nonempty. `scope` is
`user`, `session` or `agent`. Source must decode to the old fixed local JWT/host
authority with corresponding issuer kind; neither external issuer nor already
installation-bound source is eligible. Reject duplicate source tuples, blank
approvals and invalid bound destination encodings at install time.

`source_revision` is the opaque row revision returned by preview. The destination
must later equal the authenticated service-bound owner. User destination scope ID
is its verified subject; session/agent destination scope ID equals
`source_scope_id`. A mapping does not prove arbitrary session/agent access beyond
the existing owner/resource contract. A changed source requires a fresh operator
decision, not automatic revision substitution.

**NEW `POST /api/uar/credentials/migrate-principal`:** strict body
`{source_owner_key, scope, source_scope_id, provider_id, reason}`. `reason` is the
single allowlisted string `operator_approved_local_namespace_migration` for this
release. Destination, approval and source revision come only from the installed
mapping. Optional `If-Match` further binds the selected preview. Success is HTTP
200 with an immutable receipt. Missing mapping/wrong destination returns 403
`credential_mapping_denied`; source missing returns 404
`principal_credential_missing`; destination/journal conflict returns 409
`credential_mapping_conflict`; source revision mismatch returns 412
`credential_migration_precondition_failed`; store failure returns 503
`credential_migration_unavailable`. Invalid request shape returns 400.

**Existing `POST /api/uar/credentials/migrate-legacy`:** keep strict body
`{scope, legacy_scope_id, provider_id, reason}` and existing installed mapping
fields `{scope, legacy_scope_id, provider_id, destination_owner_key, approval_id}`.
Keep existing statuses/codes and reason behavior. Add optional `If-Match`; never
add body fields to a historical strict parser. An unconditioned legacy call
retains its existing semantics and must not be labelled preview-bound. Existing
free-form legacy reasons are not emitted by preview or logs; operator guidance
must require nonsecret reasons, while old receipts stay immutable.

### Metadata revisions and transactional preconditions

Introduce an opaque, nonsecret per-row `row_revision` in legacy and principal
credential storage, exposed as `source_revision` or `destination_revision` in
preview. It is a fresh canonical UUID per accepted write/recreation, independent
of secret content and timestamps. Under quiesced upgrade, initialize missing
revisions once without changing ciphertext/hints/timestamps; never initialize in
preview. Every supported writer, including system seeding, must update revision
atomically with its row. An old writer that ignores revisions is unsupported
during this upgrade; quiescence across replicas is mandatory. Missing revisions
are `revision_unavailable`, not readiness.

Preview's nullable `precondition` is a quoted strong entity tag, constructed from
the exact source kind/tuple/revision and installed mapping identity (including
destination, approval and fixed-local source revision). It contains or hashes
metadata only, never secret material. The exact opaque bytes are host-owned;
clients copy them unchanged into `If-Match`. Missing mapping/revision yields no
precondition. Mapping changes must change the tag. Accept one strong tag only;
weak, wildcard, list or malformed headers return 400
`credential_migration_precondition_invalid`. They never become unconditioned
calls. The tag is a consistency precondition, not a bearer authorization token.

First application validates mapping/caller, locks or transactionally compares
the source revision and destination vacancy, copies ciphertext/hint/timestamps,
assigns a fresh destination revision and appends the journal in one atomic
operation. Source drift with or without equal timestamps rejects; no partial
copy/journal survives. In-memory uses one coordinated critical section across
source/destination/journal; durable backends use their actual transactions.
No decrypt/re-encrypt step or secret-derived digest is required.

Legacy journal source keys stay their existing three-element JSON-array encoding.
New source keys are JSON arrays
`["principal-fixed-local-v1", source_owner_key, scope, source_scope_id, provider_id]`.
Do not include revision in the uniqueness key: a rotation must not permit a
second migration of the same source to another owner. Receipt extensions are
optional/additive `source_kind`, `source_owner_key`, `source_revision`,
`destination_revision`, `precondition`; source_kind is `legacy` or
`principal_fixed_local`. New principal receipts use `source_scope_id`; existing
legacy fields remain unchanged. Missing additions identify historical receipts.

After authorization, exact replay checks the journal first. A conditioned retry
compares against the receipt's original precondition, not the now-current source
or destination. Same original mapping/revision returns the original receipt even
if rows later change or disappear. Changed mapping/approval/revision conflicts;
no overwrite or resurrection. A historical receipt without a precondition cannot
satisfy a conditioned retry: return 412, preserving historical unconditioned
replay. A cancelled request or lost response cannot be assumed rolled back;
retry the exact authorized operation to discover its committed receipt.

### Trusted-host metadata preview

**NEW `GET /api/uar/credentials/migration-preview?limit=100&cursor=<opaque>`**.
Only existing verified trusted-host authority (`UserContext::trusted_host_id`,
backed by native launch or exact configured issuer/tenant/subject) authorizes it.
Admin role alone, settings key alone and ordinary verified principal are
insufficient. Authentication failure is 401; authenticated missing host authority
is 403 `credential_preview_denied`; unavailable store is 503
`credential_migration_unavailable`. No caller-selected database/query/owner
filter is added. Inventory is the configured store's legacy non-system and old
fixed-local principal sources plus installed mappings whose sources are missing.

`limit`: integer 1–500, default 100. Unknown query fields, invalid limits/cursors
return 400 `credential_preview_request_invalid`. Cursor is bounded (maximum
4096 bytes), opaque, integrity-protected and bound to host authority, policy and
mapping revisions and ordered source tuple. Policy/mapping change returns 409
`credential_preview_stale`; caller restarts pagination. Ordering is lexicographic
by source kind/owner/scope/scope ID/provider, using keyset continuation.

Response fields are exactly `observed_at`, `policy_revision`, `mapping_revision`,
`consistency` (literal `page_observation`), `items`, `counts`, `next_cursor`.
Each item contains `source_kind`, nullable `source_owner_key`, `scope`,
`source_scope_id`, `provider_id`, nullable `source_revision`, nullable source
`created_at`/`updated_at`, nullable `destination_owner_key`, nullable
`destination_scope_id`, nullable `approval_id`, `destination_exists`, nullable
`destination_revision`, `status`, nullable `completed_at`, nullable `precondition`.
No free-form reason or full receipt is returned.

Status is one of `unmapped_quarantined`, `revision_unavailable`, `source_changed`,
`ready`, `missing_source`, `destination_conflict`, `journal_conflict`,
`already_applied`, `already_applied_destination_missing`,
`already_applied_destination_changed`, `already_applied_revision_unknown`.
Journal status takes precedence over source/destination readiness; then missing
source, absent mapping, missing revision, source change, destination conflict,
ready. Historical receipts without a destination revision use revision-unknown
when the destination exists; a missing destination has the explicit missing
status. Never infer ready from destination absence after journal commit.

`counts` contains `page_total`, `inventory_total`, `by_namespace_scope` entries
with `namespace` (`legacy`, `local_jwt` or `local_host`), `scope`, `total`, and
`by_status` counts. These fixed namespace/scope/status dimensions keep count
metadata bounded independently of owner count. Totals describe a consistent observation for that response, including
installed missing-source mappings; they are not a frozen multi-page snapshot.
Data changes between pages can change counts or introduce earlier keys; the
operator must quiesce writers and restart inventory before cutover. A row
precondition protects application, not completeness of live enumeration.

Store APIs must project only metadata columns and aggregate counts without
loading ciphertext or hints into preview. Metadata remains operator-sensitive.
No plaintext, ciphertext, last-four hints, encryption-key fingerprints, raw
backend errors or free-form reasons enter preview/log evidence. Cancellation
performs no writes. Preview does not confer destination identity or approval.

### Quarantine and cutover

When a bound local lookup misses, a metadata-only check for the corresponding old
fixed-local scope/resource/provider blocks with `credential_migration_required`
if unresolved source exists. Recognizing possible ancestry never grants read
authority. A resolved destination stays usable. Ambiguous/unmapped data remains
blocked and quarantined; no discard or ignore-old switch is added. New rows must
not silently overwrite unresolved ownership through ordinary CRUD.

Required runbook sequence: back up both credential stores, journal, encryption
material, config and approvals; quiesce all replicas' credential writers and
consumers including provider seeding; provision deliberate mode/local UUID;
initialize metadata revisions; start controlled management host; preview and
independently approve exact mappings; apply to authenticated destinations;
compare inventory/receipts and cold-read/rotate/delete under the authorized final
gate; then admit shared traffic with exact intended grants. A preview tag does
not replace these approvals. Restore only matched database/config/key snapshots
under a compatible binary and account for post-snapshot rotations/deletions.
Old subject-only/fixed-global-local lookup is not a supported shared rollback.

## T4 — acceptance fragment and documentation inventory

This fragment follows the acceptance template. T1–T4 are stable proposed backend
task IDs for the lead's Plan ledger; no implementation assignment is issued here.
The runner below is a **NEW proposed L3 task deliverable**, absent/not run now.
Its local/arm64/amd64 manifests must exercise all declared real production paths;
merely returning zero is not acceptance. No tests or executable fixtures are
authored by this Spec task.

T4 authors fixtures and required documentation before the full-phase gate; the
boundary prohibits executing them early. Real credential and combined acceptance
runs on the candidate and each published architecture. The in-memory contract is
a source-build supplement and cannot replace image/backend observations.

| Field | Required declaration |
| --- | --- |
| Canonical identity | `uar-outstanding-issues-v1-1-1` / same OpenSpec change / T1 policy-owner, T2 source-integration, T3 migration-preview, T4 fixtures-docs; final L3 gate. |
| Source identity | Candidate UAR tree/commit and built server artifact digest captured by runner; published arm64/amd64 image digests recorded separately. Analyze baseline is not candidate acceptance. |
| Production entry point | Shipped server authenticated credential CRUD/principal/preview/migration HTTP endpoints and real run admission; embedded SDK primary/supplied-driver capture through its production host. |
| Real collaborators | Real provider HTTP peer recording account labels privately, runtime and AG-UI/persistence; in-memory contract, local SurrealKV process/restart, network SurrealDB 3.3.0 and PostgreSQL stores. |
| Boundary exercised | Verified ingress identity → host credential policy → store source selection → actual provider authorization; HTTP management → exact mapping → atomic store/journal transaction. |
| Observable result | Cross-issuer/tenant/installation isolation; exact source grant selection on every T2 seam; unchanged external/general owner bytes; bounded nonsecret preview; immutable copy/replay and cold restart; no denied provider request or partial copy. |
| Negative control | Same tenant under wrong issuer; absent shared identity; service absent; unknown supplied driver; ungranted system/config; decrypt/store failure; stale revision with equal timestamps; stale mapping/tag; concurrent conflicting copy; missing/rotated destination retry; ordinary principal preview. Each rejects through the real entry point. |
| Isolation | Runner-owned scratch config/key fixtures, installation UUIDs, dedicated backend databases/namespaces, temporary local store directories and loopback peer ports; T4 owns cleanup. No real HOME, CODEX_HOME, service databases or deployment ports are mutated. |
| Prerequisites | All parent production/config/schema/fixture/runbook work complete and lead inventory recorded; pinned server dependencies; real backends available; owner approvals represented by isolated operator mappings; sole build-target writer. Missing collaborator is BLOCKED. |
| Final local gate | Future command: `python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local.json`, cwd `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`; environment and isolated endpoints supplied by that reviewed manifest. After publication use the same runner with corresponding `arm64.json` and `amd64.json`. |
| Evidence | Pending runner receipt at phase `evidence/gates/local.json`'s declared receipt destination, containing source/artifact identity, backend, exact command/exit, observations and negative controls. Architecture manifests declare separate digest receipts and native/emulated execution. These manifests/receipts are not created here. |
| Limitations | No present runtime evidence. Memory cannot prove durable restart. API-only checks cannot prove supplied clients or all fallback paths. Emulated amd64 cannot prove native hardware behavior. No production ownership recovery or installation-ID correctness can be inferred from isolated fixtures. |

T4 must cover primary, ordinary fallback, team profile, graph, summarization,
SDK-supplied and inherited/delegated paths; config/system grants independently;
catalog aliases; permitted credential-free binding without ambient key pickup;
single-owner omitted/explicit remote compatibility; local UUID missing/restart/
signing-key rotation; migration concurrency and interrupted-response retries on
each backend; old receipts and strict legacy schema compatibility. Public receipts
record account labels, never fixture authorization headers or secret values.

Documentation deliverables: `docs/operations/principal-provider-credentials.md`
and the lead-assigned v1.1.1 upgrade notes/config examples. Include exact new fields,
breaking local-owner behavior, legacy unconditioned-call limit, metadata upgrade,
pagination consistency, approval/cutover/rollback steps and irrecoverable history.
Shared roots, schema files and API/runtime/provider files require exact
single-writer assignments in Plan. This author owns none of those production files.

## Handoff and limits

Changed files are only this artifact and the tenant-isolation delta linked above.
Interface effects are specified, not implemented. All new guards trace to the
credential boundary and the source recipes above; no unrelated production guard,
dependency update, fixture code or publication was added. Exact config fields,
API additions, source/mapping preconditions and compatibility decisions are
selected; operator UUIDs, historical ownership proof and intended fallback grants
remain operational inputs that cannot be invented.

Inspection commands used `cat`, `sed -n` and `rg` against the named repository
documents and source; `mkdir -p` and `apply_patch` authored only assigned document
locations. No verification command ran: no Cargo/test/build/lint/format/schema,
model probe or independent review gate. Behavioral implementation, backend
transaction guarantees, migration upgrade feasibility and every acceptance result
remain unverified until the complete-phase boundary. Source inspection is not
runtime or independent acceptance evidence.
