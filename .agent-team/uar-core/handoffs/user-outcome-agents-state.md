# User-outcome task 2.2 — durable agents and startup

Phase `uar-post-lifecycle-verification-readiness`; verification **CLOSED**. Source inspection and authorized implementation/documentation writes only. No test, import, format, lint, compiler, build, runner, service or review-driven runtime command has run. This handoff is implementation evidence, never product acceptance.

## Production source decisions

- `src/uar/defaults.rs`: changed built-in seeding to insert absent artifacts only. Warm startup now preserves the complete persisted default/orchestrator artifact, including operator prompt, model policy and metadata. The existing specialist/compiler preservation rule also applies to protected built-ins. Removed the stale comment claiming every restart refreshes the model selection. Storage failures now propagate to the caller rather than being swallowed per protected agent. Required seed failure startup admission is owned by the lead in `src/server.rs`.
- `src/uar/api/discovery.rs`: the authenticated session configuration handler rejects malformed `model` selectors before the durable/cache mutation. Named failure: a string without nonempty provider/model previously survived the echo/cache but became `None` in `into_run_policy`, silently losing the explicit operator override.
- `src/uar/domain/agent_store.rs`: no modification needed. `is_protected_agent_id` includes `default-agent` and `orchestrator-agent`; `delete_agent` rejects before persistence access.

## Public deletion inventory (source trace)

`src/server.rs` mounts `DELETE /api/agents/{id}` to `api::discovery::delete_agent`, which invokes `domain::agent_store::delete_agent`, translating Protected to 403. `api::administration_capabilities` advertises this same endpoint as `agents.delete`; this is metadata, not an independent mutator. Registered MCP/native skill tool discovery and compiler publication contain no other agent-definition delete endpoint. The compiler uses create/replace, not delete. Persistence providers implement low-level delete and `persistence/scoped.rs` delegates it, but these are host storage interfaces, not independently exposed management handlers. Embedded administration exposes agent upsert/read/list; no public delete method was found. No delete/reseed workaround is used.

## Startup and maintenance authority

`server.rs` initializes settings, seeds configured providers, hydrates registry from persisted settings, seeds agents, then installs the run manager and retention owner. `SettingsManager::initialize_with_governance_default` preserves API-edited values (updated_at/source tracking); providers hydrate from stored configs/default-provider setting. Embedded startup invokes the same built-in seed function and already propagates its failure. `RunManager::spawn_retention_sweeper` owns real run/session retention, wakes on activity/expiry, and opt-in `sessions.persisted_ttl_secs` drives durable expired-session deletion. Memory expiry has a separate lifecycle-owned `memory-expiry` worker. U21 must use a real owned maintenance fixture and durable effects; elapsed time or a fabricated trigger is not proof.

## Harness authoring

Awaiting task 2.1 shared source/API handoff before authoring dependent `scripts/acceptance/cases_agents.py`. Intended cases: U03 prompt effect before/after update and true restart/custom deletion; U04 explicit refusal and unchanged durable built-ins before restart plus useful prompt availability; U20 second genuine model, session precedence, rejected mutation preservation and restart; U21 empty/warm seed idempotence and real maintenance receipts followed by prompt/restart.

## Remaining gaps and unverified claims

All runtime behavior is unverified because the gate is CLOSED. No compile or import claim is made. The startup readiness behavior on seed failure needs lead integration. The harness requires actual fixture observers and both approved durable adapters; absent evidence must BLOCK, never PASS. No guard was added beyond the malformed model admission check tied to the observed conversion defect.

## Task 2.1 source API handoff accepted; dependent module authored

`outcome_harness_core` handed off `core.py` and `cases_prompt.py` source interfaces before dependent module authoring. `scripts/acceptance/cases_agents.py` now exports four three-repetition durable cases:

- `U03.custom-agent`: mounted persisted create/read/list/patch; fresh private system-prompt markers drive actual genuine inference before/after update; true child-process restart preserves exact saved artifact; custom delete produces persisted 404 and prompt rejection, remaining deleted after another restart.
- `U04.protected-agents`: both protected IDs refuse the sole exposed DELETE path with 403; exact persisted artifacts compared before any restart (prevents reseed masking); useful genuine arithmetic prompt immediately and after restart; catalog contains exactly one of each; unrelated custom deletion control succeeds.
- `U20.effective-configuration`: distinct second genuine provider/model; revision-fenced REST default setting; stale-revision mutation refusal without last-good change; live provider default applies to omitted-model inference; persisted session model overrides global default; malformed model admission refusal preserves last-good session config; both actual selections survive cold restart. Restores primary default within owned fixture.
- `U21.startup-maintenance`: requires empty-store provenance; all five built-in seeds occur exactly once initially and after warm startup; edits protected default agent prompt/metadata, proves private prompt effect, cold restart exact preservation; requires actual `run-retention` persisted TTL deletion, then true restart durable absence, untouched saved agent and useful private prompt. Restores the run-owned built-in edit afterward.

Shared helper checks protocol terminals, effective agent/model, immutable agent revision, settled provider call accounting and separately sourced genuine provider provenance. Case predicates additionally grade exact independently calculated output; nonempty text cannot pass. Missing provider, durable adapter or observer evidence raises BLOCKED. No module was imported or executed.

### Unfinished executable fixture dependency (reported to lead)

The case module requires concrete frozen backend observers; generic command callbacks alone are not implementation completion:

- `startup_initial`: bound run_id/backend, `empty_before_first_start=true`, actual `storage_identity` from owned initial store observation.
- `startup_maintenance`: receives worker=`run-retention`, run-owned random expired_session_id and storage identity. Must create the actual expired persisted session under the owned principal/store, allow the configured opt-in production retention worker to delete it, and return matching run/backend/storage/worker/session identity, actual nonempty `persisted_before`, `persisted_after=null`, `deleted_by_worker=true` and a concrete `worker_receipt`.
- `startup_maintenance_read`: after child-process restart, return matching expired_session_id/storage identity and `persisted_record=null` from that durable store.

No arbitrary maintenance endpoint, forced deletion masquerading as worker action, elapsed-only proof, or live-handle reconstruction is admitted. Ownership of executable observer fixture files was not included in this packet and must be assigned before task 2.2 is marked source-complete. Other shared observers (`inference`, readiness, cleanup) are task 2.1 foundation dependencies.

## Verification status at handoff

**CLOSED**. Commands used were source reads (`cat`, `sed`, `rg`, `ls`), authorized file writes (`apply_patch`, heredoc, mkdir), and team messages. No verification command executed. The harness is unparsed/unimported/unrun; Rust is uncompiled; none of U03/U04/U20/U21 has runtime acceptance. Required startup failure readiness integration remains lead-owned. No speculative guards were introduced; malformed selector rejection traces to the observed lossy conversion. No unrelated production source changed.

## Executable startup observer source dependency authored

Lead expanded ownership to `scripts/acceptance/observers_startup.py`, its `.README.md`, and ONLY the `list_expired_sessions` section of `src/uar/persistence/providers/postgres.rs`.

- `observers_startup.py`: actual local Postgres psql adapter (credentials in environment, source-derived schema/key/columns) and loopback Surreal HTTP `/sql` adapter (same transport already used by retained repository fixture scripts). Both enforce the exact run-derived isolated schema/DB, read physical storage identity, capture all application-table initial counts before first boot, preserve an exclusive private receipt, and read actual persisted session metadata. The maintenance prepare action ages only the genuine prompt-created target row's last_activity, returning actual mutation metadata atomically; it never creates/deletes a session or agent. The observe action requires a durable absent row AND a new exact owner/session production retention log marker. The read action reads absence again after true restart. Log excerpts are bounded and public output retains closed metadata/digest only. Arbitrary upstream error strings are withheld, typed failure retained and unrecovered cause stated.
- `observers_startup.README.md`: source contracts, exact hook/actions, reference-only config schema, derived scope algorithm, private state setup, binary/config freezing, opt-in retention and worker logging, actual mutation effects, cleanup ownership, and the explicit unverified limits. This is not an executable fixture or authority.
- `postgres.rs::list_expired_sessions`: observed defect was `split_once("::")` against actual `session_storage_key` length-prefixed keys. Replaced with data.owner_id/data.id plus exact reconstructed key equality. Unknown/mismatched records are not parsed speculatively. Source ownership of this provider section is released to lead.
- `cases_agents.py` U21 now creates the aged target through genuine inference, calls maintenance prepare, runs another genuine prompt to emit actual retention activity, then observes the actual worker effect. It uses the actual returned session ID rather than an invented empty row.

Shared foundation owner supplied prestart `startup_initial_capture` dispatch and private named actual server stdout/stderr paths. Root supplied metadata-only per-session retention success logging in the existing production deletion branch. Root was additionally notified of the exact TTL-only configuration defect: the manager returned before persisted TTL when idle eviction/cap were both zero; that source correction is root-owned and must be reconciled in the final inventory.

### Current integration requirements, not missing observer implementation

Task2.9 must bind/freeze these concrete source observers into both actual backend fixtures with their real environment references/executables/store preparation and server configuration. The observer configuration is immutable and uses `schema_scope`/`database_scope = run_sha256_20`; actual run scope is derived at runtime rather than placing a future random run ID in frozen bytes. Preparation/server wrappers must use the same algorithm. These are normal exact-packet preparation requirements, not success labels. Generic readiness/admission/inference/session/cleanup observers remain shared foundation responsibilities. All required observers now have authored startup/maintenance implementations; none has been executed.

**Final verification status: CLOSED.** No tests, imports, compiler probes, lint, formatting, builds, database queries, worker launches or acceptance runners were run. Source inspection and authorized writes only. Production behavior, SQL compatibility and case execution remain entirely unverified until explicit operator authority. No additional guard was added without a real storage/credential boundary or the named malformed-selector/TTL-key failure scenario. No publication or third-party resource mutation occurred.

Shared-source integration handoff: the lead's `observers_runtime.py` may import guarded `observers_startup.py` to reuse `run_scope`, `session_storage_key`, `Store`, actual scoped `counts`, exact-owner `session_data` and metadata-only `session`. `session_data` validates the actual serialized owner/logical ID against the requested physical partition and returns full persisted messages without writes. Root owns store prepare/cleanup and generic observer source. Source APIs and SQL fields/namespace guards were sent explicitly; no imports were run. Root now owns defaults/discovery and the repaired PostgreSQL section; this teammate retained case/startup observer authoring only until final handoff.
