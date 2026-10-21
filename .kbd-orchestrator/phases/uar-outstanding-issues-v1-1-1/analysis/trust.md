# Credential authority and migration analysis

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Analyze. Date: 2026-10-15.
Assignment: `uar-trust-tools`; sole writable artifact is this file. Production
implementation, external research and verification are outside this assignment.
The full-phase verification boundary remains **CLOSED**.

## Recommendation and uncomfortable case

Add an explicit host credential policy with `single_owner` and `shared` modes.
Bind local credential owners to a stable operator-provisioned installation
identity, enforce shared fallback policy before every client is captured, and
extend the existing migration mechanism to include existing fixed-local principal
rows. Preserve the v2 principal table, ciphertext format, existing legacy mapping
API and copy-plus-journal transaction semantics.

The uncomfortable case is a well-intentioned global change to
`CredentialPrincipal::storage_key`: that key also owns sessions, knowledge,
approvals and durable schedule/followup grants. Changing it merely to repair
provider credentials would make unrelated existing state inaccessible. Another
dangerous shortcut is accepting an old local principal row because its subject
matches the current caller: fixed local authorities contain no installation
provenance, so that match does not prove ownership.

Choose a credential-service-specific bound owner, derived from an already
verified principal plus trusted installation policy. Keep the general principal
encoding unchanged in this phase. This is a narrow storage/credential boundary,
not a new global identity system. General local identity isolation outside
credentials remains a separate limitation and must not be represented as fixed.

## Evidence and source-grounded reproducers

Read `AGENTS.md`, the authoritative waypoint (revision 2926, next command
`/kbd-analyze uar-outstanding-issues-v1-1-1`), `versions.toml`, team README/map/tool
and skill policies, assessment and its issue-380 companion, local gotchas, the
required security/Rust skills, and the following source. Compass metadata still
names source commit `ba12845138104d3c8c3b8bca8bc7c5be24004e91`; it is not this
phase's assessment baseline `91a091d647fe2ce63651c07140172aba2c6c6302`. No current
call-graph or runtime conclusion is derived from it. Current source supplies the
claims below. No upstream behavior is asserted and no external queries ran.

| Source | Observed mechanism |
| --- | --- |
| `src/uar/security/claims.rs`, `CredentialPrincipal::from_verified_context` and `storage_key` | Issuerless verified identity uses `urn:uar:local:jwt`; local launch uses `urn:uar:local:host`; absent tenant encodes as `local`. Context consistency and anonymous rejection already exist. |
| Same file, `UserContext::session_owner_key`, `ScheduleOwnerGrant::restore`, `FollowupOwnerGrant` | General identity keys own other durable state; schedule restore explicitly recognizes the old local authorities. |
| `src/uar/security/credentials/resolver.rs`, `resolve_with_context` | Principal session, agent, user misses unconditionally call `resolve_system`; errors propagate. |
| `src/uar/runtime/manager.rs:597`, `apply_credential_layer` | Absent service retains config; absent principal resolves system; a total miss retains configured key. |
| `src/uar/runtime/manager.rs:5153`, team profile loop and model capture around 5333 | Primary and team fallback resolution exist; configured ordinary fallback and supplied primary-driver paths also reach client capture. |
| `src/uar/runtime/turn/bindings.rs:201`, `RunModelBindings::capture` | A supplied primary driver is accepted directly; ordinary fallback construction calls `fallback_llm_config` then `build_driver_captured`. Filtering only `cfg.api_key` does not govern these seams. |
| `src/uar/security/credentials/mod.rs` and `src/server.rs:1110` | Encryption-key presence controls optional service creation. Legacy mappings come from `CREDENTIAL_OWNER_MAPPING_FILE`; there is no explicit credential tenancy policy. |
| `src/config.rs:679`, `SecurityConfig` | Existing deployment profile, exact trusted-host principals and JWT issuer/audience/verifier policy are useful boundaries; they do not define credential fallback permission. |
| `src/uar/security/credentials/principal_store.rs` | Memory/Surreal/PostgreSQL stores isolate v2 rows, copy legacy ciphertext atomically with a journal, preserve source/timestamps, reject conflicts and retain idempotent receipts. Only legacy-table sources are accepted. |
| `src/uar/api/credentials.rs` | CRUD and `/principal` capture the general principal directly; migration matches an installed mapping to the authenticated destination. |
| `src/uar/api/providers.rs`, provider verification; `src/uar/settings/manager.rs`, `protect_provider_credentials` | Administrative provider workflows read or seed protected system credentials directly. Storage protection is not authority for ordinary user runs to consume them. |

These are **unexecuted source reproducer recipes**, recorded before any fix:

1. Two installations using issuerless verified JWTs with equal subject and absent
   tenant construct identical owner keys. Connect both to the same credential
   database: the principal lookup tuple has no installation discriminator. A
   different installation namespace cannot be established by the current key.
2. Supply a valid principal with no session/agent/user provider record and a
   configured house key. `resolve_with_context` tries the system row; if absent,
   `apply_credential_layer` keeps the configured key. Repeat with no verified
   principal: its direct system branch or unchanged config permits the same
   selection. There is no explicit shared-policy refusal in either branch.
3. Store a local principal credential using today's key, then change the local
   issuer constant to an installation-specific value. Ordinary reads seek a new
   owner. `/migrate-legacy` cannot recover that row because it only reads
   `provider_credentials`, not `provider_principal_credentials`.
4. Add a resolver-only denial but leave a supplied primary driver or ordinary
   configured fallback unchanged. `RunModelBindings::capture` still accepts the
   client or constructs the fallback outside that resolver decision. This is a
   required seam to close, not a demonstrated runtime bypass of a proposed fix.

## Proposed policy and configuration

**Every key/type/endpoint introduced in the remainder is a NEW PROPOSAL, not an
existing interface.** Existing configuration names are explicitly identified.
The lead must carry the selected names into an OpenSpec delta and plan before
implementation. No dependency or protocol pin change is required.

Propose a `security.provider_credentials` object in trusted startup config:

| New proposed field | Semantics |
| --- | --- |
| `mode` | `single_owner` or `shared`; omission retains existing single-owner compatibility. Shared mode is explicit opt-in. No inference from deployment profile, first caller, database engine, number of users or JWT roles. |
| `installation_id` | Nonempty canonical UUID required to bind local/issuerless principal credentials; provision once by the operator and keep stable across restarts/replicas of that installation. External issuer-qualified owners do not require it. It is a nonsecret namespace identifier, not proof of possession. |
| `shared_fallbacks` | Default empty array. Each exact `{tenant_id, provider_id, source}` entry allows one verified tenant to use one host-owned provider source; `source` is `system` or `config`. No wildcards, empty identifiers or request-supplied entries. |
| `principal_mapping_file` | Optional operator-owned path for the new fixed-local-v2 migration mapping format. Keep existing `CREDENTIAL_OWNER_MAPPING_FILE` and its strict legacy schema unchanged. |

The existing `UAR_SECURITY__...` environment/config mechanism can expose these
fields when integrated by the shared-config owner. Do not introduce a second
independent policy source or silently ignore contradictory settings. Existing
`CREDENTIAL_ENCRYPTION_KEY` still supplies encryption and existing
`CREDENTIAL_OWNER_MAPPING_FILE` still supplies legacy mappings.

Lead-selected compatibility default: absent `mode` preserves historical
single-owner behavior. Explicit `single_owner` remains valid for remote as well
as local deployments; transport exposure does not establish tenancy. This mode
means callers admitted by existing ingress rules may consume the configured
house account. It is not per-person isolation and does not weaken ingress
verification. Operators must explicitly select `shared` to receive the shared
tenant/fallback restrictions. Document that compatibility limit prominently.
Missing installation ID fails only local/issuerless principal binding with an
actionable safe error, never implicit old-namespace lookup. It does not prevent
startup or issuer-qualified credential use merely because encryption is enabled.

| Context/result | `single_owner` | `shared` |
| --- | --- | --- |
| Verified issuer + tenant + subject | Resolve isolated principal scopes. | Resolve isolated principal scopes. |
| Verified issuerless token or authenticated local launch | Bind local credential owner to configured installation; absent tenant is local only. | Refuse credential use. Require a verified configured issuer and nonempty signed tenant. Never synthesize tenant from deployment config or headers. |
| External verified issuer with missing tenant | Permit existing local compatibility encoding. | Refuse before lookups/fallback, even if the provider has a fallback entry. |
| Anonymous/synthetic/inconsistent identity | No principal CRUD or migration. Existing locally admitted house use may remain. | Refuse credential-bearing run binding. Fallback permission does not mint an identity. |
| Principal scopes miss | System then configured key in existing order, subject to migration-required guard. | Only exact tenant/provider source entries permit fallback. Otherwise return a typed denial/missing-credential outcome. |
| Store/decrypt/identity/migration failure | Stop; never reinterpret as a miss. | Stop; never reinterpret as a miss. |
| Service absent/encryption key absent | Existing env/config compatibility as above; no principal store lookup. | Policy still applies. Config fallback requires the exact grant; system source requires an available service. Never equate `None` with permission. |

Keep `single_owner` session/agent/user precedence. In shared mode the same
precedence applies before explicitly authorized system and config fallback. A
system lookup miss can proceed to config only if config is independently allowed;
a system error cannot. Provider IDs mean the exact resolved catalog identity,
including aliases, not an untrusted display name or guessed model prefix.

Do not use clearing an API key as a denial: constructors may resolve environment
credentials later. Carry a host-owned resolution result describing the selected
source and exact provider; denied paths return before client construction.
Explicit request-bound credentials continue through their existing host grant and
exact provider/endpoint validation, with shared identity eligibility enforced.
Credential-free local inference remains possible only through an explicitly
admitted credential-free binding that cannot acquire ambient credentials.
Unknown/opaque supplied drivers cannot establish that property by their label.

Freeze the policy with root host model bindings. Descendants reuse the captured
authorized subset; they cannot broaden policy or reconstruct a missing binding.
Apply the decision to ordinary/graph roots, every fallback, team profiles,
summarization and reused/supplied clients. A legacy SDK constructor with no policy
can retain documented single-owner behavior; it must not be usable to
construct a shared host silently. Define explicit shared construction instead.

Administrative provider verification is a distinct authorized host operation:
retain protected settings/auth boundaries and exact provider target binding. Do
not treat ordinary fallback permission as admin authority, or treat an admin
verification receipt as permission for all users to consume its system key.

## Stable authority without a global principal rewrite

Propose a private-field `CredentialOwner` capability created by `ProviderService`
or its separately retained policy handle from a verified `CredentialPrincipal`.
The caller cannot deserialize one from an HTTP body or stored row. The new type
retains the verified identity and the credential storage identity separately.
Principal CRUD/resolver/store boundaries must require the bound owner, preventing
an accidental raw-principal path around installation binding.

For issuer-qualified identities keep existing owner bytes unchanged. For local
JWT and local-host identities use the existing length-framed encoding with
distinct authorities derived from `installation_id`, for example proposed
`urn:uar:installation:<uuid>:jwt` and `urn:uar:installation:<uuid>:host`. Keep the
two kinds distinct. A captured local delegation must bind identically to its
root; stripping host privilege must not change the credential owner. General
`CredentialPrincipal`, `ActorOwner`, session ownership, schedule grants and KB
namespaces keep their existing bytes.

Bootstrap is deterministic from explicit host configuration: validate the
operator-provisioned UUID and derive the two authority strings from that exact
value. Startup never generates a replacement ID. Provision the ID in host
configuration and back it up with the encryption key,
database and mapping approvals. Never derive it from JWT secret material, process
ID, hostname, a path, a user, first login or a fresh random value on each boot.
JWT key rotation must not rotate credential namespace. Replicas share the same
ID only when they are intentionally one installation. A cloned deployment is
not automatically entitled to adopt the original installation's credentials.

Do not bind the entire credential database to one installation ID: disjoint
installations may share that database. No new singleton metadata record or
automatic ID-drift detector is proposed. Stable configuration and operator backup
discipline preserve the ID. A changed valid ID denotes a different credential
namespace and cannot recover another installation's rows by a subject match.
Missing/invalid ID blocks local principal binding, and unresolved old fixed-local
sources block fallback as described below. Automatic detection of accidental
replacement of a valid ID is not claimed. Memory mode has only process lifetime
and must remain labelled ephemeral.

`GET /api/uar/credentials/principal` should return the service-bound credential
owner in its existing fields, so operators obtain the actual migration
destination. Its local owner value intentionally changes; externally issued
owners remain stable. If the service/policy cannot bind an owner, return a safe
unavailable/identity error rather than a plausible old destination string.
CRUD request bodies and masked response shapes remain unchanged.

## Existing fixed-local-v2 migration and nonsecret preview

Keep `provider_principal_credentials` as the v2 store and retain all legacy rows
and existing receipts. The key encoding's `v1` prefix is independent of the v2
table discriminator; do not confuse the two version concepts or rewrite rows.

Extend migration with a separately parsed exact mapping containing proposed
fields `source_owner_key`, `scope`, `source_scope_id`, `provider_id`,
`destination_owner_key`, and `approval_id`. Only fixed-local namespaces are in
scope for this new source type. Require operator evidence for the entire source
and destination; subject equality supplies no authority. Ambiguous preexisting
collisions cannot be recovered from ciphertext alone and remain quarantined.

Propose `POST /api/uar/credentials/migrate-principal` alongside the current
legacy endpoint. The request selects the exact source tuple and nonsecret
reason; it cannot choose destination or approval. The authenticated destination
must match the installed mapping. User scope must target the verified subject;
session/agent scope preserves the exact source resource identifier and bound
owner. Do not add a caller-selected remapping of arbitrary sessions or agents.

Use the same transactional algorithm on all three stores: validate exact source,
check destination vacancy and source migration journal, copy ciphertext/hint and
timestamps, append receipt, commit together. No decryption/re-encryption is
required. Preserve the original source as quarantine. A replay returns the
original receipt and never overwrites rotated values or resurrects a deletion.
Changed destination/approval, concurrent conflicting mapping, or existing
destination fails. Keep the old legacy journal key format exactly unchanged;
new principal-source journal keys need a disjoint version/type tag including
source owner, scope, scope ID and provider. Do not reuse only the legacy
three-part source key. Extend receipt data additively with source-kind/owner
fields; old receipts remain readable and immutable.

Add a typed migration-required result for unresolved fixed-local rows that
would otherwise appear to be a new-owner miss. Recognizing a possible old source
is sufficient to block fallback, never sufficient to read/adopt its secret.
The preview and approved migration resolve that condition. For this phase,
unmappable sources remain quarantined and the affected local owner/provider
operation remains blocked; add no automatic retirement or ignore-old-row switch.
Already bound owners can continue. Do not silently make an existing credential
appear absent and charge the house account. Existing issuer-qualified v2 rows
remain addressable without local namespace migration or an installation ID.

Propose `GET /api/uar/credentials/migration-preview`, restricted to verified
trusted-host authority using the existing native launch/exact service-principal
boundary. A role string such as `admin` alone is insufficient. It exposes only
operator-installed mappings and selected credential metadata; never accept an
arbitrary database query or let ordinary users enumerate other owners. This is
an additive management operation, not a normal credential list.

Preview includes:

- Totals by source namespace/scope: inventory, mapped, unmapped/quarantined,
  ready, already-applied, missing-source, destination-conflict, journal-conflict.
- Exact nonsecret source tuple and configured destination tuple, approval ID,
  source timestamps, destination existence and migration receipt status.
- Separate status for `already_applied_destination_missing` or changed
  destination revision. Never classify a missing destination as permission to
  replay the copy. Equal old and new subject names are not a ready mapping.
- Snapshot time and policy/mapping revision identifiers. The preview is advisory;
  application rechecks the exact rows and journal transactionally after quiescing
  writers. A preview token is not an approval or a lock.

Do not emit plaintext, ciphertext, last-four hints, encryption-key fingerprints,
raw backend errors or free-form secret-bearing reason fields. Identities and
provider metadata are still operator-sensitive; restrict access and bound/page
the result. Obtain inventory through typed metadata-only projections. Do not
load full credential values merely to serialize a redacted output. Cancellation
of preview causes no writes. A dropped migration response does not imply
rollback: retry the exact mapping to retrieve its committed receipt.

## Cutover and rollback contract

1. Back up old and principal credential tables, migration journals, encryption
   material, trusted config and approval mappings through authorized operational
   procedures. Record versions and source inventory without secrets in receipts.
2. Quiesce every legacy/v2 credential writer and in-flight credential consumer
   across all replicas, including administrative settings protection/seeding.
   A single stopped replica does not establish quiescence.
3. Select deliberate mode and provision the stable installation ID when local
   principal credentials are involved. External issuer-qualified credentials
   need no local ID or namespace copy. Start only the
   controlled management host, preview both legacy and fixed-local-v2 sources,
   and obtain independently verified exact destination identities. Unmapped rows
   remain visible in operator inventory and quarantined.
4. Install mappings consistently, apply through authenticated destination APIs,
   and retain atomic receipts. Retrying an uncertain outcome is safe only for the
   exact mapping. Never edit journal entries to resolve a conflict.
5. At the authorized full-phase gate, prove reads/rotation/deletion/restart and
   actual model binding with the intended account and no denied fallback. Compare
   inventory/receipt counts. Then enable shared service traffic with explicit
   fallback entries only where the operator intends a house account.
6. Roll back by quiescing new traffic and restoring the matched database/config/
   key snapshot under a supported compatible binary, or keep the new binary in
   controlled recovery. Do not run an old subject-only or global-local-authority
   binary as a shared rollback. Source retention is recovery evidence, not
   permission to re-enable its lookup. Address rotations/deletions after the
   snapshot explicitly before restoring; never resurrect them silently.

## Proposed ownership, fixtures and documentation

These are proposed implementation assignments; the lead must issue exact
single-writer packets before edits. This analysis makes no production assignment.

| Surface | Proposed single writer and required outcome |
| --- | --- |
| `src/uar/security/credentials/{mod,resolver,principal_store,store}.rs`, new local policy module if needed | Trust: bound owner, policy result, migration/preview interfaces and store behavior. Keep all implementations of the trait under one writer. |
| `src/uar/security/claims.rs` | Trust only after lead assignment, and only a narrow trusted identity accessor if required; no global storage-key rewrite. |
| `src/uar/api/credentials.rs` | Trust with explicit cross-domain assignment: bound CRUD identity and authorized preview/new migration endpoint. |
| `src/config.rs`, `src/server.rs`, `src/lib.rs`, config/schema examples | Lead or one designated shared-root writer: config validation, stable policy construction and propagation even with no encrypted service. |
| `src/uar/runtime/manager.rs`, `src/uar/runtime/turn/bindings.rs` | Runtime owner: every root/fallback/supplied/inherited client respects the same source decision; preserve provider timeout and thread authority contracts. |
| Provider admin/verification and settings paths above | Their assigned owner, after trust handoff: distinguish administrative system use from user fallback; bind exact provider and safe errors. |
| Additive store schema/migration files | State owner under a serialized handoff from trust; preserve existing migrations and legacy journal interpretation. No concurrent edits to store implementation. |
| `tests/credentials_api_integration_test.rs`, `tests/support/credential_process.rs` | One fixture owner: extend real-process authenticated HTTP/resolver/provider-peer scenarios and backend coverage. Existing shared-house cases must explicitly opt in. |
| `docs/operations/principal-provider-credentials.md`; release upgrade notes and OpenSpec delta | Lead-assigned docs writer: config break, preview/cutover/rollback procedures, limits and exact commands selected by plan. |

Required fixture inventory, to author before the phase gate and execute only
after it opens:

1. Issuer/tenant isolation and existing CRUD, precedence, error, replay/conflict
   cases retained. Same local subject in installations A/B cannot share a bound
   owner; restart and JWT key rotation retain the installation namespace.
2. Shared missing/blank tenant, issuerless identity and synthetic identity fail
   safely. Absent service does not bypass shared policy; absent mode retains the
   documented single-owner default, including remote deployments. Missing ID
   blocks local principal use but leaves external principals usable. Exact system/config grants allow only
   the configured tenant/provider/source, including catalog aliases. Store and
   decrypt failures never trigger fallback.
3. Primary, ordinary fallback, team, graph, SDK-supplied driver and delegated
   child paths use an allowed source or fail before provider effects. Capture
   provider-peer authorization in private fixture memory; public receipts contain
   account labels only. Include no-credential local binding without ambient key
   pickup and denied supplied opaque driver.
4. Both legacy and fixed-local-v2 migrations across memory, local SurrealKV,
   shared SurrealDB and PostgreSQL: exact approval, wrong installation, conflicts,
   concurrency, interrupted response, restart, ciphertext/timestamp preservation,
   rotated/deleted destination replay, immutable old receipts and disjoint journal
   keys. Explicitly select ignored backend cases; report unavailable backends.
5. Preview has accurate counts/conflicts and no secret/hint material, rejects
   ordinary principals, and does not mutate source, destination or journal.
   Stale preview must not overwrite a concurrently changed destination.
6. Global session and schedule/followup owner bytes are unchanged by credential
   binding; a scoped delegation binds to the same credential owner as its root.

Reuse the whole integration command recorded by assessment once authorized:

```text
cargo test --locked --no-default-features --features server-full,postgres-backend --test credentials_api_integration_test -- --include-ignored --test-threads=1
```

It is **not executed** here and does not alone prove all model/SDK pathways.
The lead's plan must attach the remaining real-path scenarios to the relevant
whole targets and schedule one serialized full-phase gate. No new command flags,
CLI migration commands or passing results are claimed.

## Concrete decisions to carry into Plan

Lead-selected decisions: service-local credential owner; explicit stable UUID
only for local/issuerless principal binding; omitted mode retains single-owner
compatibility; shared mode is explicit opt-in independent of deployment profile;
exact tenant/provider/source fallback grants; separate fixed-local mapping format;
additive host-only metadata preview; both source kinds preserve atomic copy/journal
and quarantine. No database-global installation singleton is added. Existing
issuer-qualified credentials remain usable without a local ID or migration.
Generic principal/session/KB/schedule identity bytes remain stable. Each guard
addresses #380's explicit credential requirements or an existing authentication,
storage or client-construction boundary.

Keep the existing journal table and legacy source keys unchanged. New source keys
are tagged and include the complete principal source tuple; receipt fields are
additive. Irrecoverably ambiguous sources remain quarantined; the affected local
lookup blocks rather than silently falling through. Preview uses the existing
verified trusted-host authority; no new administrative CLI is selected. Existing
public diagnostic code fields remain intact. Convey safe stage/reason/reference
through the existing message and structured logs; add no AG-UI event fields.

No material design choice is left as a pending option list. Operator-specific
inputs remain unknown: actual installation UUIDs, historical ownership evidence,
permitted fallback pairs and deployment-specific backup/quiescence procedures.
Do not invent them. The implementation must refuse missing inputs where required
while preserving the compatibility paths above. Supplying those inputs is an
operational prerequisite to real migration, not permission for this analysis to
change a live deployment. No automatic detection of a wrong but valid replacement
installation ID or recovery of historically overwritten collisions is promised.

## Handoff and verification limits

Changed path: only
`.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/analysis/trust.md`.
Interface effects: none implemented; the tables above describe proposals.
Unrequested production changes, guards, dependency updates and publications: none.

Inspection used `cat`, `sed`, `rg` and a read-only Python JSON projection of graph
metadata. Initial assumed paths for the companion assessment and SDK source did
not exist; subsequent file/source discovery supplied the actual paths. Broad
graph output was truncated, so it was not treated as evidence; the bounded build
metadata projection identified the old graph revision. No tests, compiler
checks, formatting, lint, builds, model probes, review gates or external queries
ran. No runtime reproducer or backend acceptance result is claimed. Production
behavior, migration concurrency, final API compatibility and all fixture outcomes
remain unverified until planned implementation and the full-phase gate.
