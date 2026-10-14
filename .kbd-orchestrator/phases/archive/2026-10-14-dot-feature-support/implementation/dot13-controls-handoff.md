# Dot13 memory controls handoff — source authored

Phase: Dot Execute. Production implementation and bounded real-store fixture are
authored. Nothing in this handoff is a compilation, runtime or acceptance claim.
No canonical waypoint, task ledger or OpenSpec checkbox was changed here.

## File ownership and delivery

- `src/uar/memory/controls.rs` (new): strict owner retention contract, durable
  policy DTO, opaque host-only CaptureAdmission, MemoryService control/status and
  purge methods, explicit unsupported behavior without the Surreal feature.
- `src/uar/memory/controls_store.rs` (new): transactions on actual pinned
  SurrealStorage::db; additive policy/control tables; exact database-clock capture
  anchor and immutable retry identity; epoch/revision fencing; monotone shortening;
  protected provenance; read suppression; deletion/history scrubbing; resumable
  purge; atomic source-fenced summaries with flattened lineage and inherited expiry.
- `src/uar/memory/owned.rs`: all ordinary direct/search/list/hybrid/history reads
  check controls; explicit save/update/delete and bulk deletion use controlled
  host writes; queued capture uses opaque admission; compression/TaskStreams use
  controlled summaries and filtered contexts. Protected metadata cannot be forged.
- `src/uar/memory/service.rs`: initialize controls and resume purge at startup;
  existing TTL maintenance resumes durable purge rather than raw SDK delete.
- `src/uar/memory/auto_capture.rs`: final `Option<CaptureAdmission>` argument;
  None means no capture, never reacquire a queue capability.
- `src/uar/memory/migration.rs`: stable-key controlled import prevents deleted
  import recreation; replay is filtered; original quarantined source is unchanged.
- `src/uar/memory/mod.rs`: controls module export.
- `src/uar/api/memory.rs`: controls_router with policy, export, direct record,
  history and non-content control status routes.
- `src/uar/api/memory_admin.rs`: admin list/stats/search filter visibility; bulk
  deletion goes through controlled per-record host deletion.
- `src/uar/admin/memory.rs`: trusted admin get/update/delete honor controls;
  explicit shared creation uses controlled storage, preserving host authority.
- `src/uar/memory/mcp_server.rs`: MCP policy inspection/change methods; existing
  ordinary operations already enter OwnedMemory.
- `src/uar/tools/memory.rs`: new read-only MemoryPolicyTool. Existing native
  save/recall/delete/update/history inherit controlled adapter behavior.
- `tests/dot_memory_controls.rs` (new): real production MemoryService/store and
  native scenarios in joined local processes and two shared processes. No mocks.
- `tests/dot_memory_owner_contract.rs`: only an explicit UntilDeleted policy opt-in
  before the existing assertion that automatic capture is nonempty; root authorized.
- `openspec/changes/dot-13-memory-controls/operator-memory-controls.md` (new):
  operational contract, endpoints, profile and rollback limitations, deferred gate.

No changes to vendor, dependencies, config, PersistenceLayer/provider modules,
manager, server, SDK callers, UI, workflow scripts or canonical statuses by this
teammate. Other dirty work was preserved. Root separately applied server wiring.

## Root composition contract (reported applied by root)

1. Nest `api::memory::controls_router()` under `/api/memory`. Its relative paths
   are `/policy`, `/records`, `/records/{id}`, `/records/{id}/history`, and
   `/records/{id}/control`. Existing legacy GET/POST `/api/memory` remains.
2. Register `tools::memory::MemoryPolicyTool::new(Option<Arc<MemoryService>>)`;
   tool name is `memory_capture_policy`, effect ReadOnly.
3. Before spawning stream-end capture, await
   `svc.capture_admission(&ctx2) -> Result<Option<CaptureAdmission>>`. On error,
   report a non-content diagnostic and use None. Pass the result as the final
   argument to `auto_capture::capture_from_stream_end`. Admission must precede
   enqueue; a worker must not reacquire it. Root retains the original UserContext.
4. Existing startup MemoryService::new and TTL worker perform purge; no new
   detached worker or build target was added.

## Decisions and mechanisms

`memory.auto_capture=true` is an available host capability, not consent. Owner
policy defaults off in existing and new stores. A false host switch is exposed
and refuses enabling. Explicit enablement chooses exactly 7/30/90/365 days or
until-deleted; unknown/mixed/missing enablement choices reject.

Capture and policy mutation write the same owner policy row. Disable increments
consent_epoch; queued old admissions remain refused after re-enable. Retention
revisions without revocation use current policy at commit. Captures use a stable
hash of complete owner/agent/session/role/content, preserve the first database
capture timestamp, and refuse tombstoned/expired replay. Existing per-record
expiry is shortened durably in the policy transaction and cannot be extended.

Explicit records do not need automatic consent and do not inherit its retention.
Legacy records are not automatically classified. The raw SDK's semantic dedup
write is bypassed because it could mutate a prior explicit record or refresh a
removed capture. This changes near-duplicate explicit creation into independent
explicit records rather than silently modifying older records.

Deletion first commits an owner/content-free tombstone, removes edit-history
rows and clears record metadata including embedded imported history. It also
revokes flattened summary descendants and removes their history. Physical
record/vector cleanup then resumes idempotently at startup/TTL. All public
owner and admin reads filter tombstones/expiry; errors fail closed. A raw trusted
host storage handle remains database maintenance authority, not an export API.

Summaries fence source versions/control expiry and retain all ancestor identities;
source removal or expiry suppresses derived content before cleanup. Summary
retention is never later than its source bounds. Queued capture and late update
cannot recreate removed records: they conflict on policy/control/memory writes.

Dot12's original unsuffixed quarantine remains separate operator migration
material. Imported owner content/history is removed, and replay cannot restore
it. No silent original-source destruction or PostgreSQL memory parity is claimed.

## Evidence and explicitly deferred work

Executed commands were source reads/searches and implementation/document writes
only. The actual MemoryStorage trait, SurrealStorage::db/store/write methods,
private DbMemory/schema and TaskStream paths were read. Context7 provided current
SurrealDB transaction documentation. No Cargo, tests, compiler probes, checks,
format, lint, validators, real-path probes or review gates ran. There is no pass
receipt and no claim that these files compile.

Authored command, to run only after the entire expanded Dot production inventory:

`cargo test --locked --no-default-features --features server-full --test dot_memory_controls -- --nocapture --test-threads=1`

The fixture requires isolated durable UAR_DOT_MEMORY_SURREAL_URL with optional
USER/PASS variables matching the existing Dot12 fixture, plus the local embedding
model. It verifies authored scenarios through joined local processes/shared
processes; fixture clock placement avoids sleeping days. The test has not run.

Remaining unverified: SQL syntax/schema compatibility, Rust signatures/derives,
actual transaction conflict behavior, owner HTTP/MCP transport composition,
summary races, exact timing and interruption during index cleanup, all affected
backend feature profiles and the required customer build. Source-authored
fixtures and documentation are complete; independent review waits for the full
phase boundary. The uncomfortable thing is the gap between the transactional
source intent and still-unexecuted production evidence.

## Scope and guards

No unrelated feature was added. Opaque admission/epoch, stable capture identity,
owner checks, protected provenance, current expiry and source-version fences,
lineage revocation, tombstones and idempotent purge trace to the explicit Dot13
consent/deletion/race requirements and the existing host storage trust boundary.
No speculative retries, lease, new queue, timeout or database engine was added.
The fixture alone uses bounded waits to fail a missing cross-process handshake.

Please append these decisions and the verification deferral to the phase's
canonical progress/memory record when recording root integration; this teammate
has not competed for the lead's authoritative ledger or append-only memory file.
