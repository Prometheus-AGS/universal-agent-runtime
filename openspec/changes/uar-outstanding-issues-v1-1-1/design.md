# Design

## Context

See `proposal.md` for the problem and scope. The phase's `analysis.md`, A01–A11
decisions and three source investigations are the design inputs. The baseline is
`91a091d647fe2ce63651c07140172aba2c6c6302`; no implementation result is inferred.
The reported deployment has an explicit repository-agent token ceiling and no
visible remote durability attestation. Its live persisted agent and erased inner
cause remain unverified. #384 timeout inheritance is independent and retained.

The runtime already has durable budget reservation, principal credential v2
storage, atomic migration journals, owner-scoped ingestion and fenced team
recovery. The change extends those boundaries. `versions.toml` keeps Liter 2.1.1
at its pinned commit and SurrealDB/types/server 3.3.0. No vendor patch, new library,
new tenancy framework or kernel write capability is selected.

## Goals / Non-Goals

Goals: preserve safe binding causes; enforce terminal truth; complete native
Anthropic framing; close credential policy and migration seams; make the selected
upgrade path understandable; produce reproducible local and published-artifact
evidence for v1.1.1.

Non-goals: changing generic principal keys, restoring ambiguous raw-subject
access, privileged legacy KB import, force/TTL team takeover, reducing budgets,
blind durability attestation, external Dot qualification, UI redesign, production
cluster mutation, or pretending the skill-version warning has a proven cause.

The uncomfortable limit is operational: a correctly refusing runtime can remain
unusable with an unattested store, missing KB originals or unavailable fencing
proof. Safe diagnostics and isolated acceptance do not establish customer recovery.

## Decisions

### Typed diagnostics stay at the host composition boundary

State provides a captured safe durability classification through the existing
persistence capability boundary; runtime maps it and originating typed errors
into an allowlisted binding stage/category. No string matching or arbitrary
`anyhow` formatting classifies the error. Preserve existing ordinary/team public
codes and the RUN_ERROR code/message wire shape. Safe stage/category/reference
is rendered in the existing message and structured logs, without raw URLs,
provider payloads, credentials, prompts or effective model configuration.

This avoids both alternatives identified in analysis: opaque generic errors
cannot diagnose #385, while logging an opaque constructor chain can expose keys
before the scrubber knows them. A diagnostic reference is correlation, not an
endpoint granting access to secret detail. It does not create a new receipt API.

The existing admission ordering remains: budget and owner eligibility before
provider/tool effects; current unknown liabilities remain reserved. Reverification
belongs to its separate target-admission/configuration contract and is not an
accounting repair. API upload 503 and dispatcher logs use the same captured safe
eligibility reason without changing the durable-before-202 promise.

### Runtime completion and provider framing remain separate

The ordinary manager tracks observed orchestrator completion. EOF with no
completion and no cancellation becomes a failure before success settlement;
existing cleanup errors, joined cancellation, interrupted history and reservation
ownership retain their precedence. Provider markers or transport-close envelopes
are insufficient proof that orchestration succeeded. The working orchestrator
terminal guard and graph paths remain; no common state-machine rewrite is needed.

Native Anthropic receives a UAR-owned incremental input adapter around the pinned
Liter decoder. It handles exactly one initial BOM and CR/LF/CRLF across byte
boundaries while preserving strict UTF-8, bounded incomplete-frame state, field
semantics and EOF validation. It must process a final CR as a real delimiter,
without manufacturing an additional blank line at EOF. A pending CR cannot
strand an otherwise complete event indefinitely; normalization may emit its LF
immediately and retain only whether to swallow a following LF.

Classify a syntactically valid JSON top-level string type before deserializing
known kinds. Unknown well-formed kinds with matching/empty SSE event name produce
no state change. Invalid JSON, absent/non-string type, malformed known kinds,
name mismatch, data after message_stop and missing terminal remain failures.
Preserve tool/usage handling and error propagation. Do not add `eventsource-stream`
or modify the Liter submodule: candidates cand-001/cand-003 supply reuse/reference,
and cand-002 was rejected as unnecessary replacement scope.

### Credential policy binds authority before any client construction

The exact proposed config/API contracts live in `spec/trust-contracts.md` in the
phase and the tenant-isolation delta. The operator owns
`security.provider_credentials`: mode, optional local installation UUID, exact
shared fallback grants and fixed-local migration mappings. Omitted mode retains
single-owner compatibility, including remote deployments. Shared mode is explicit;
a verified external issuer and nonempty signed tenant/subject are mandatory.
A grant matches issuer, tenant, resolved provider identity and source; including
issuer prevents a tenant string collision from granting another issuer's account.

Construct an internal bound credential owner from verified identity plus host
policy. External issuer owner bytes remain stable. Local JWT/local-host owner
bytes use separate installation-qualified authorities; the UUID is provisioned
once by the operator, stable across replicas/restarts and backed up. Missing UUID
fails the affected local credential operation, not unrelated external-principal
startup. No database-global installation singleton or global principal rewrite
is introduced. Sessions, KB, approvals and schedules retain existing owner bytes.

Carry a typed source decision to every primary/fallback/team/graph/supplied-driver
binding. A denied decision stops construction; clearing an API key is insufficient
because a constructor might pick up environment credentials. Missing credential
service does not bypass shared policy. Stored/decrypt errors are not misses.
Opaque prebuilt clients without acceptable host-owned source provenance cannot
satisfy shared authority. Descendants inherit narrowed root authority; request
metadata or a manifest cannot expand it. Administrative provider verification and
settings storage retain their existing authority and do not imply user fallback.

### Migration preserves source and explains incomplete ownership

Reuse encrypted v2 storage and atomic copy+journal. Add an explicit fixed-local
source namespace mapping, disjoint journal identity and metadata-only row revision
for conditional preview/apply. Keep legacy bodies/mappings compatible; an optional
conditional header can bind legacy apply to a preview without changing the strict
body schema. Unconditioned legacy calls retain their existing contract and cannot
claim preview-binding protection. Store upgrades initialize revisions without
reassigning owners or re-encrypting values; credential writes advance revisions.
Plan selects a NEW local host operation,
`universal-agent-runtime credentials initialize-revisions --writers-quiesced`,
as the explicit upgrade entry point. Additive schema installation does not backfill
rows. The command requires the operator acknowledgement, opens only the configured
credential backend and exits without starting listeners, providers or MCP. The
flag acknowledges the documented backup/quiescence procedure; it does not prove
exclusion. Missing acknowledgement refuses mutation. Repeating initialization
leaves already assigned revisions and all credential values/owners unchanged.
Only safe counts and allowlisted errors may be emitted. This command, its docs
and positive/negative installed-image fixtures are inside T3/L2/T4 and the same
full-phase implementation boundary.

Preview is trusted-host-only, paginated, metadata-only and read-only. It reports
counts/status/conflicts, not keys, ciphertext, hints or backend errors. Pagination
is not a transaction-wide snapshot; record that limitation, quiesce writers and
recheck source revision and destination/journal in the copy transaction. Exact
replay returns its original receipt; it cannot recreate deleted destinations or
overwrite rotations. Fixed-local unresolved rows are quarantined and may block
house fallback; a matching subject only detects ambiguity, never grants ownership.

The precise quarantine/disposition representation is part of T3's selected
contract, not permission to bypass a missing mapping. Backends are memory,
SurrealKV/shared Surreal and PostgreSQL; schema and store changes use a serialized
state-to-trust handoff. No destructive migration or live database action occurs
in this stage.

### Recovery uses existing supported operations

The supported KB path is authenticated create/reuse, reupload original source,
wait for durable indexing, retrieve and ground a run as the exact verified owner.
Legacy records remain unchanged. It does not preserve old IDs/chunks or recover
missing originals. Qualified storage evidence precedes any attestation. Team
recovery uses existing joined quiesce or authenticated predecessor-exclusion
proof and exact reclaim; no Pod-presence heuristic or single-replica exception.

The missing-version startup warning gets troubleshooting guidance that captures
resolved pack root and exact sanitized path/error. Current builtin loading
permits absent SKILL.md version; no speculative bulk edit or compiler relaxation
is planned. New causal evidence would require a scope revision before the full
implementation boundary.

### Whole-phase implementation, authorized preparation, then immutable publication

`verification.md` defines the new L3 local runner interface and complete group
inventory. R5/T4/S4 author reusable real-process fixtures; L3 integrates black-box
observations against the candidate and the selected published-image paths. Native-source tests
are separate evidence. A published run must never rebuild or replace UAR source.
Use the production Dockerfile and existing deployment workflow; no workflow change
is selected. Version bump is limited to the runtime Cargo package/lock entry and
required release documentation.

After the complete implementation inventory, separately authorized image and
qualified-supervisor preparation, isolated artifact review and the serialized
six-group local program must succeed before source/tag publication. Both
published children must pass streams/recovery after their own authorized
same-image preparation. Record index,
child digests, source/workflow identity and native/emulated execution. Failed
published acceptance withholds success; never silently move the tag. Issue
reconciliation preserves v1.1.0 history, actual #385 recovery limits and #382
external dependencies. This does not revive cancelled Dot qualification.

## Ownership and serialized integration

| Writer | Exclusive implementation responsibilities to assign in Plan |
| --- | --- |
| Runtime | manager/cost_budget/turn bindings; native Anthropic driver and new adapter; runtime and MCP scenario fixtures/docs. Trust supplies policy interfaces and state supplies eligibility types. |
| Trust | credentials policy/bound owner/resolver/store/API and migration preview; credential fixtures/docs; existing provider-admin/setting authority integration only through explicit lead handoff. |
| State | persistence eligibility, knowledge/dispatcher diagnosis; additive schema files; durable-ingestion/knowledge recovery docs and fixtures. State is sole writer of durable-ingestion.md and incorporates runtime wording by handoff. |
| Lead | config/server/module exports, runtime version/lock, L3 runner integration, release records, complete inventory, canonical KBD state and one build-target writer. |

The new adapter module export and credential policy configuration require the
lead's shared-file handoff. The trust-owned credential store implementations
must not be edited concurrently by state. Plan assigns exact paths and sequence
for each task; this table is not permission for overlapping writes.

## Risks / Trade-offs

- Local credential namespaces change → fail actionably, retain sources, require
  exact approved mapping, document the compatibility break and test cold readback.
- Shared mode opt-in retains legacy house behavior by default → state that default
  honestly; do not market it as multi-user isolation. Explicit shared grants are
  narrow and issuer-qualified.
- Metadata revision/schema support expands the credential change → use additive
  schema and existing atomic transaction boundaries; preserve old receipt reads.
- A framing adapter can double-count CRLF or alter buffer behavior → specify
  split equivalence, terminal controls and cumulative incomplete-frame limits;
  final gates run through the real provider HTTP path.
- Positive fixture attestation could hide the reported refusal → separate
  unattested negative profile and qualified durable positive profile with actual
  cold restart evidence; neither attests know-me.
- Acceptance orchestration might certify a source binary instead of an image →
  prohibit rebuild/source mounts in published mode and bind all results to the
  exact inspected child digest and fixture revision.
- Review/validation is deferred → artifact completeness is not semantic approval;
  V1/V2 remain mandatory after the full implementation inventory.

## Migration Plan

1. Complete code, schemas, fixtures, configs and docs; freeze the inventory and
   perform local review/acceptance. No live deployment changes follow implicitly.
2. Operators inventory and back up credential rows/journals, encryption material,
   mappings and trusted configuration; quiesce all relevant writers/consumers.
3. Provision a local installation ID only where needed; preview exact legacy or
   fixed-local mappings; apply approved conditional copies and retain receipts.
   Unmapped/ambiguous records remain quarantined. Keep shared traffic disabled
   during cutover; enable it only with the selected explicit grants and evidence.
4. Use authenticated KB reupload and existing qualified team recovery as needed,
   after the deployment-specific prerequisites are established and authorized.
5. On rollback, quiesce traffic and use a compatible binary with a matched
   database/config/key snapshot. Do not re-enable subject-only shared lookup;
   account for rotations/deletions since backup before restoration.
6. Publish v1.1.1 and attach immutable acceptance/upgrade receipts. Product release
   and actual customer recovery remain separate statuses.

## Evidence inputs still pending

Actual installation IDs, approved principal mappings and fallback grants are
operator data, not design choices to invent. Live persisted agent, storage
backing, original KB source availability and qualified predecessor exclusion
remain recovery prerequisites. Their absence blocks the corresponding recovery
claim rather than changing this specification. No unresolved dependency/stack
choice is deferred to implementation.

## Prior lessons applied

The Spec hook's prior-context file reports that the learning-recall library is
unavailable; no remote recall success or retrieved knowledge gap is claimed.
Local append-only `.prometheus/gotchas.md` supplies the relevant lessons:
2026-10-12 records that a healthy remote endpoint does not attest durability and
that per-slice testing violated the full-phase boundary; 2026-10-13 records that
raw-subject identity cannot adopt private rows. These constrain the eligibility,
migration and acceptance contracts here. The prior release reflection forbids
reviving cancelled Dot qualification. Generic Rust tier rules and skill review
suggestions do not override the repository's full-phase boundary.


## Execute correction — operator-selected integration scope

On 2026-10-16 the operator explicitly limited final verification to real
integration of known scenarios supported by this phase. References above to
seven groups, the full historical Kubo program or broad source validation are
superseded. The original six-group published repetition is additionally superseded
by the 2026-10-17 function-first amendment below. Production behavior remains fully specified;
no production requirement is deferred. Internal SDK seams use real provider/store
integration, and no unit-only, lint, schema or feature-matrix sweep is acceptance.
Full implementation, required fixtures/docs and L4 still precede every gate.

## Function-first certification amendment — 2026-10-17

The child plan is the current implementation sequence: FC1–FC5 complete all
contracts, adapters, fixtures, runner/schema integration and whole-phase inventory;
FC6 separately prepares the image and fresh qualified supervisor state; FC7 is
authorized independent review; FC8 is authorized local acceptance and operator
timing disposition; FC9 hands back to parent V2/P1–P3. The operator gate remains
CLOSED throughout partial implementation and never opens merely from inventory.

Use the existing runner and five release adapters for six groups. Local acceptance keeps all six groups
and four native host/SDK/memory supplements. Published acceptance selects exactly
streams/recovery on each immutable child. No architecture-specific credential
migration, budget refusal, compiler restriction or MCP failure claim is inferred
from those two groups; those outcomes are certified on the local candidate only.
Both published children retain durable owner isolation and qualified supervisor
recovery. Missing genuine qualification is a blocker, not removable metadata.

The controlled provider must derive its answer from both a unique user prompt
and the enabled attached skill's unique overlay. Fully assembled public SSE,
AG-UI and persisted answer text must equal that answer exactly. A disabled skill
remains eligible but yields the existing disabled activation failure; its overlay
and unique dependencies stay absent while ordinary prompting may continue.
Record client admission/first-content/terminal timing and peer receive/write
durations without subtracting unrelated monotonic epochs. The operator explicitly
accepts the measured timing or requires investigation; no numeric SLO or real-model
quality claim is invented.

FC4 authors a closed `image_preparation: "prebuilt"` manifest mode with
`prepared_image_id` and hashed `image_preparation_receipt`. The function-first
acceptance invocation inspects and uses that exact image; it cannot build, pull
or silently replace it. A prior image-bound supervisor receipt is invalid after
source/fixture edits. FC6 builds/inspects the local candidate under separate exact
instructions, stages real queued/active predecessor work with an operator-qualified
controller, performs the separately authorized physical boot transition, and
records a fresh distinct replacement fence on the same image. Each local/published
recovery run consumes its own fresh proof; no previous reclaim is replayed.
Published children require separate authorized pull/inspection and equivalent
fresh staging after P1 supplies actual digests. Image identity is not controller
qualification. FC5's published manifests remain non-executable templates until
those receipts exist. This design adds no kernel write or alternate authority.

The uncomfortable limit is the deliberately smaller per-architecture observation
set and unmeasured timing. Label consolidation is not evidence of fewer requests;
the child disposition records actual removed loops/restarts separately.
