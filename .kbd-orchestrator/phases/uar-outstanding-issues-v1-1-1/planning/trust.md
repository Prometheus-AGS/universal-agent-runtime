# Trust implementation plan

Phase: `uar-outstanding-issues-v1-1-1`. Stage: **Plan**. Date: 2026-10-16.
Author: `uar-trust-tools`. Stable tasks: **T1, T3, T4**; T2 remains runtime-owned.
Full-phase verification boundary: **CLOSED**. This is collaborative plan authoring,
not isolated review, implementation, migration execution or acceptance.

## Authority and scope

The selected behavior remains exactly the phase `spec/trust-contracts.md` and
`openspec/changes/uar-outstanding-issues-v1-1-1/specs/tenant-isolation/spec.md`.
OpenSpec `tasks.md`, `design.md` and `verification.md` define the complete parent
boundary. Waypoint revision 2932 identifies this phase and `/kbd-plan` as next.
This plan adds file assignments and internal handoffs, not a new requirement.
Only this planning file is written by this task. All production assignments below
are proposals for the lead's exclusive ownership ledger before Execute.

Applied skills: `.agents/skills/agent-runtime-security/SKILL.md`,
`.agents/skills/rust-best-practices/SKILL.md`, and the required
`/Users/gqadonis/.codex/skills/prometheus-rust-workspace/SKILL.md` entrypoint.
Private host-created capabilities carry verified authority; explicit fallible
results distinguish denial, missing rows and storage failure. Secret material
stays out of metadata/debug/receipt paths. Repository phase rules override all
earlier check recommendations. No dependency, vendored source or pin changes.

The uncomfortable case is that a correct policy resolver can coexist with an
unguarded supplied or fallback client. T1 alone cannot deliver isolation. Another
is a ready preview followed by a source rotation: timestamps and an unconditioned
legacy migration do not bind the preview to the eventual copy.

## Source-grounded reproducers before implementation

These are source recipes, not executed exploit or acceptance results. The team
map records an older Compass graph and parse gaps; it is navigation evidence.
Current source establishes the following mechanisms.

| Recipe | Existing source evidence | Planned closure |
| --- | --- | --- |
| Give two local installations equal issuerless subject/tenant and one credential DB. | `src/uar/security/claims.rs:199` assigns fixed local authorities; `storage_key` at 235 encodes them. | T1 binds only credential ownership to the provisioned installation UUID. Generic principal bytes remain unchanged. |
| Miss principal scopes with a house credential; repeat with encrypted service absent. | `resolver.rs:56` ends at `resolve_system`; `manager.rs:597` returns original config when service is absent and retains it on misses. | T1 exact source decisions, T2 before client capture, L2 independent policy construction. |
| Deny only resolver resolution and supply an existing driver or ordinary fallback. | `src/uar/runtime/turn/bindings.rs`, `RunModelBindings::capture`, captures supplied primary and fallback clients. | T2 requires host-verifiable provenance at every capture, including inherited subsets. |
| Preview a source, rotate it with unchanged timestamps, then copy. | `CredentialRecord` in `store.rs:59` has no revision; principal migrations compare mapping/destination but not a preview revision. | S3 additive metadata, T3 atomic compare/copy/journal and immutable replay. |
| Migrate an old fixed-local principal row. | All `migrate_legacy` implementations in `principal_store.rs` read only `provider_credentials`. | T3 explicit fixed-local mapping, separate source kind and disjoint journal key. |
| Read legacy source while another memory writer changes it. | `InMemoryCredentialStore` has separate `rows` and `principals` locks; migration holds principal lock while reading legacy rows. | T3 coordinates source, destination and journal under one critical section for conditional migration. |

Existing migration receipts and exact legacy source keys are compatibility
inputs. No source recipe authorizes reading production secrets or changing a
live deployment. NativeTool/McpRegistry descriptions and annotations remain
untrusted data and confer no credential ownership, source grants or approval.
Existing tool effect classification, policy, exact-resource approval,
cancellation and redaction remain under the trusted host; this plan does not
rewrite MCP execution. Runtime R4 owns the selected outcome fixtures.

## Exclusive file ownership for Execute

`Existing` means the path was inspected or located in this checkout. `NEW` means
the proposed file does not yet implement a callable interface. One trust writer
owns the entire T1/T3/T4 sequence; task labels do not permit concurrent writers.

| Task / writer | Exact path | Change |
| --- | --- | --- |
| T1 trust | NEW `src/uar/security/credentials/policy.rs` | Policy types/config validation, exact source grants, safe denial classes and host-only `CredentialSourceDecision`. |
| T1 trust | NEW `src/uar/security/credentials/owner.rs` | Private `CredentialOwnerBinding`, credential-only local authority derivation and strict source/destination key interpretation. |
| T1 then T3 trust | Existing `src/uar/security/credentials/mod.rs` | Export selected types, compose independent policy with optional encryption service, install strict mappings and host-only migration/preview operations. |
| T1 then T3 trust | Existing `src/uar/security/credentials/resolver.rs` | Bound-owner lookups, ordered independent system/config decisions, typed denial and unresolved-local-source quarantine. |
| T3 trust | NEW `src/uar/security/credentials/migration.rs` | Strict fixed-local mapping, preview DTOs, preconditions/cursor metadata, bounded status/count construction and receipt comparison helpers. |
| T3 trust | NEW `src/uar/security/credentials/maintenance.rs` | Explicit local host revision-initialization operation, backend-only connection and safe counts/errors. |
| T1 signature transition, then T3 trust | Existing `src/uar/security/credentials/principal_store.rs` | Require bound owner, metadata-only queries, conditional fixed-local/legacy copy transactions and historical receipt compatibility for memory/Surreal/PostgreSQL. |
| T3 trust | Existing `src/uar/security/credentials/store.rs` | Revision-aware stored row decoding/writing and explicit initialization; unified memory coordination; metadata projections without ciphertext/hints. |
| T3 trust, explicit cross-domain assignment | Existing `src/uar/api/credentials.rs` | Bound CRUD/principal state, strict new routes/query/header handling and exact safe HTTP mappings. |
| T4 trust | Existing `tests/credentials_api_integration_test.rs` | Whole production target cases, including memory/SDK supplements and actual durable backend observations. |
| T4 trust | Existing `tests/support/credential_process.rs` | Owned runtime/provider processes, trusted isolated configuration, backend preparation, restart/join and redacted observation helpers. |
| T4 trust | NEW `scripts/release/uar_1_1_1_credentials.py` | Installed-image credential scenario adapter; no runtime execution or import side effects during authoring. |
| T4 trust | Existing `docs/operations/principal-provider-credentials.md` | Exact policy/API compatibility, metadata upgrade, preview/apply/cutover/rollback and limits. |

`encryption.rs` needs no algorithm change. `claims.rs` general storage encoding is
preserved. If binding needs access to verified issuer kind, request a narrow
read-only accessor from the L2 shared security writer; do not infer local identity
from an issuer string alone or widen principal construction. All existing direct
principal-store call sites must migrate through the file's assigned writer; a
compatibility overload taking a raw principal cannot bypass the binding.

Lead L2 is sole writer for `src/config.rs`, `src/main.rs`, `src/server.rs`,
`src/uar/security/mod.rs` if export changes are needed, shared bootstrap/state
composition and `src/uar/security/claims.rs` if that accessor is needed. L2 also
owns explicit `src/uar/api/providers.rs` and `src/uar/settings/manager.rs`
administrative integration. Trust supplies typed handoffs and documentation text,
not parallel patches. Lead owns runtime package version, release notes,
`example.config.yaml`, runner/manifest scaffolding and any shared fixture module.
T2 runtime owns manager/capture/SDK/summarization/graph/team/delegation callers;
its own plan enumerates exact paths. No trust edits to runtime sources.

## T1 → T2/L2 interface

Proposed Rust names are `CredentialTenancyPolicy`, `CredentialOwnerBinding` and
`CredentialSourceDecision`. The first two match the state Spec vocabulary. These
are internal implementation names, not new serialized grants. The policy is
constructed from trusted `security.provider_credentials` fields exactly as
specified: mode, optional canonical installation UUID, exact fallback entries and
optional principal mapping file. Preserve current encryption/legacy mapping
environment inputs and existing nested config loading; add no alternate policy
environment switch.

Owner construction accepts verified captured identity and host policy only.
Store methods receive the resulting capability, never a request string masquerading
as an owner. It retains the general verified principal alongside the separate
credential key. External keys retain their exact bytes; local host/JWT authorities
remain distinct. Captured delegation uses the same root credential identity.

The decision records owner eligibility, resolved catalog provider, exact endpoint,
selected source and captured policy revision. Sources are principal
session/agent/user, system, config, existing exact explicit request grant, or
explicitly credential-free. The decision carries no plaintext in its metadata;
only the existing credential-bearing value holds secrets. Policy/identity/storage/
decrypt/migration failures are distinct from absence and must not become fallback
warnings. A system grant cannot grant config; denied system access makes no
system-row query. No encrypted service still requires the independent decision.

T2 consumes decisions before constructing or accepting primary, ordinary fallback,
endpoint-profile, team, graph, summarization and SDK-supplied clients. A credential-
free decision must prevent ambient key pickup. Opaque supplied clients fail shared
admission without host-verifiable provenance. Descendants only reuse or narrow
captured authority; already admitted runs retain their subset. L2's administrative
verify/seeding path preserves its separate existing authority and exact target;
it cannot mint ordinary run fallback permission.

Runtime consumes `CredentialSourceDecision` directly and obtains the authorized
connection without interpreting raw policy or fallback details. The decision
retains provenance alongside the resulting config so that clearing/changing
config fields cannot stand in for authorization. Each ordinary fallback and
supplied primary accepted inside `RunModelBindings::capture` also requires its
individual decision. Narrowing through `for_policy` never re-resolves authority.

API state must carry policy independently of optional `ProviderService`; otherwise
the current `Option<Arc<ProviderService>>` state cannot implement the absent-service
contract. T3 defines its state object and L2 passes it at router construction.
The Spec's existing 401/503 behavior and added allowlisted error codes are retained.

## S3 → T3 schema and transaction handoff

Proposed S3 ownership is NEW `migrations/20261016000000_credential_revisions.sql`
and NEW `migrations/surrealdb/credential_revisions.surql`. Lead assigns the single
writer for existing `src/uar/persistence/providers/surreal_host.rs` registration.
Existing PostgreSQL `providers/postgres.rs:54` uses `sqlx::migrate!("./migrations")`;
do not rewrite historical migration files or their checksums. S3 does not edit
`credentials/store.rs` or `principal_store.rs`.

The schema handoff contains nullable historical `row_revision` storage for both
credential tables, a canonical UUID string format, projection column names and
upgrade ordering. Every new/updated/recreated row receives a fresh nonsecret UUID
atomically with the write, including system seeding. T3 owns typed row decoding,
all store writers and transactional revision initialization. Absence decodes
compatibly but preview reports `revision_unavailable`; never manufacture a
revision during reads. Initialization preserves ciphertext/hints/timestamps and
does not rewrite owners.

Existing journal is JSONB in PostgreSQL and SCHEMALESS in Surreal. Optional receipt
fields are additive: source kind/owner/revision, destination revision and
precondition; new principal receipts use source scope ID. Historical rows and
receipt keys stay untouched. Keep the three-element legacy source key and use
the exact specified five-element `principal-fixed-local-v1` key for fixed-local
sources; revision never enters uniqueness identity.

First application compares installed mapping, bound destination, source revision
and destination vacancy atomically with ciphertext copy and journal insertion.
Replay authorizes first, then returns the original journal receipt even when
source/destination changed or disappeared. Conditioned replay checks original
precondition, never today's source. Historical unconditioned replay remains valid;
historical receipts cannot satisfy a new conditioned retry. No decrypt/re-encrypt
or secret-derived fingerprint is needed. Cancellation/lost response is an unknown
commit result resolved by an exact authorized retry, not assumed rollback.

**Operational separation:** additive schema installation may use existing startup
registration. Revision backfill must be an explicit operator action after all
credential writers/consumers are quiesced, including other replicas and provider
seeding. Normal startup is not evidence of quiescence. The lead-selected NEW local
operation is `universal-agent-runtime credentials initialize-revisions --writers-quiesced`.
L2 owns `Cli`/`Command` and dispatch wiring in `src/config.rs`/`src/main.rs`; T3 owns
`credentials/maintenance.rs` and the store initialization operation. The explicit
acknowledgement is mandatory but does not prove exclusion: backup and quiescence
remain the operator's responsibility. Open only the configured credential backend
using existing connection mechanisms; do not start listeners, providers or MCP.
Return safe counts/errors only. This concretizes the selected explicit supported
upgrade, adds no task and requires no migration HTTP route, automatic backfill,
policy field or standalone environment switch. Lead records the matching Spec
design/verification/L2 clarification before Execute.

## T3 operation order

1. Install validated policy/mappings and independent API state. Bind every
   credential destination; reject unsupported identity before lookup/mutation.
2. Consume S3 schema contract; implement revision-maintaining writers and explicit
   quiesced initialization with no secret mutation. Coordinate memory state under
   one lock and durable operations under actual backend transactions.
3. Implement exact mapping authorization and both transactional migration paths,
   preserving legacy strict body/reason/status semantics and optional `If-Match`.
   Reject malformed/weak/wildcard/list tags rather than treating them as absent.
4. Implement metadata-only trusted-host preview with exact Spec DTO/status/count
   fields, keyset ordering, limit 1–500 and integrity-protected cursor bounded to
   4096 bytes and bound to host/policy/mapping revisions. Query projections/counts
   never load ciphertext/hints. Preview is read-only, including cancellation.
5. Enforce unresolved old-local quarantine on missed bound lookup and ordinary
   CRUD; successful bound destinations remain usable. Ancestry detection grants
   no ownership, and no ignore-old bypass is added.
6. Finish T4 fixture code/docs and pass inventories to L3/L4. No execution occurs
   after any individual step or task.

## T4 fixture and documentation boundaries

The installed-image adapter implements the lead-defined
`run_group(context) -> structured result` interface. Context supplies the owned
runtime/image endpoint, private infrastructure/configuration, evidence/resource
roots and lifecycle helpers. Group results enumerate required scenario IDs,
positive/negative observations, contact counts, backend, redacted artifacts and
pass/failed/blocked/interrupted status. It may not choose arbitrary shell commands
from the manifest, build UAR, patch/mount replacement binaries or claim missing
backends passed. Importing the module starts no processes or requests. L3 alone
owns runner dispatch, shared context/result definitions and manifest identity.

T4 covers all T2 seams in the source target, with real public equivalents in the
image adapter; SDK/internal-only behavior remains explicitly source evidence.
Required dimensions include issuer/tenant/installation collisions, local UUID
absence/stability/signing-key rotation, shared eligibility, absent service,
independent config/system grants, aliases, store/decrypt errors, opaque supplied
clients, inherited narrowing and credential-free ambient-key prevention. Allowed
controls prove intended provider account selection without exposing keys.

Migration cases include both sources, all preview statuses/counts/pagination,
malformed/stale cursor and tags, changed mappings/revisions/equal timestamps,
concurrent destinations, interrupted responses, source retention, old receipts,
destination rotation/deletion and restart. Use actual SurrealKV, shared SurrealDB
and PostgreSQL; memory is supplemental. Privileged preview denial is separate from
destination-authorized apply. Secret canaries must stay absent from public output,
logs and receipts. Each backend gets a separate declared observation inventory.

Runtime R5 owns the combined failed-MCP/incomplete-stream driver. T4 supplies
credential setup/assertion helpers, not a second combined runner. Trust does not
edit `tests/bauar_full_harness_cursor.rs` or runtime fixture files concurrently.
The lead or R5 integrates helpers under explicit ownership; cleanup uses L3's
owned process/network/volume manifest and joins before cold readback.

The existing principal-provider runbook must include exact new config/API fields,
single-owner default limits, local-owner compatibility break, strict mapping
formats, conditional versus historical unconditioned apply, immutable replay,
metadata upgrade operation and quiescence, pagination's page-observation limit,
backup/approval/cutover and matched-snapshot rollback. Lost originals/rotations
cannot be reconstructed; old subject-only/fixed-local shared access is not a
supported rollback. L2 receives exact setup/upgrade text for its release notes
and example config. Real operator IDs, mappings, grants and recovery proof are
operational inputs, never fixture values copied into deployment guidance.

## Dependencies, deferred evidence and exit inventory

Order: L1 ownership → T1 policy/binding interface → T2 runtime integration and L2
shared wiring; S3 schema contract → T3 store/API work; T1/T2/T3/S3 handoffs → T4
complete fixtures/docs → L3 runner integration → L4 complete phase inventory.
T1 may author interfaces alongside S3 schema work because their files are disjoint.
The same trust writer serializes its shared credential files. T3 cannot claim
implementation completeness until the explicit initialization entrypoint is usable.

L4 must list each path above, all migrated callers, optional-service wiring,
every backend transaction/revision writer, both migration sources, preview,
quarantine, initialization, all scenario adapters and required docs. Missing
operator fixtures/prerequisites are disclosed and cannot become skipped passes.
No uncertain production item opens the boundary.

After L4 only, V1/V2/P2 use the exact OpenSpec verification contract. Deferred
credential source argv is
the two exact cases registered in L3: native SDK supplied credentials and
real HTTP/in-memory conditional migration. The runner supplies the locked
server-full/postgres/in-memory feature set with `--exact --test-threads=1`;
no whole-target or include-ignored sweep is selected.
The lead's serialized full local runner then exercises all six phase image groups,
including credentials/combined; each published architecture repeats all image
groups under its observed digest. No command in this paragraph has run here.

Plan-stage observed work: `cat`, `sed -n`, `rg` and bounded file inspection read
the named artifacts/source; `apply_patch` authored this file. No product edit,
test, check, build, formatter, validator, model probe, independent review or
publication ran. No unrequested production guard was added. Planned guards map
to the source recipes and selected real credential trust boundary. Source
inspection does not establish compilation, backend atomicity, runtime denial,
installed-image behavior or migration correctness; all remain unverified.


## Lead consolidation — 2026-10-16

S3 is the sole writer of the two named migration assets and surreal_host.rs
registration. T3 follows S3; T2 follows T3's finalized store/source-decision
interface. T3 completes its backend/maintenance operation and source handoff;
L2 subsequently makes the CLI command externally usable. T3 does not depend on
L2 completion, so there is no dependency cycle. End-to-end usable behavior is
mandatory at L4; a T3 source handoff alone cannot open verification. The exact
CLI, negative control and preservation contract are now recorded in the OpenSpec
design, tenant-isolation delta and verification contract.
