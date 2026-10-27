# Repair009 memory boundary source plan

Date: 2026-10-25. Phase: uar-post-lifecycle-verification-readiness. Stage: user-outcome-implementation. This packet is source planning only, not review certification or acceptance.

## Lock and ownership

Verification gate CLOSED. No tests, lint, format, typecheck, Cargo resolution/check/build, compiler/import probes, packaging, review-driven verification, services, providers, acceptance scripts, or external gates ran or are authorized by this packet. Classify every subsequent command as source inspection, authorized implementation/documentation/memory writing, or verification. Gate opening belongs to the lead and operator under AGENTS.md. Other writers share this checkout; preserve their edits. This task owns this handoff only until a followup explicitly grants source ownership.

Waypoint inspected: revision497, planRevision133, implementation144/168; current task2.9. Exact next command remains STOPPED P0.10 runtime feature-profile compilation, with P0.005 receipts preserved. Bounded repair009 authorization is supplied by the lead task packet; this plan does not alter canonical position.

## Observed source defect and contracts

`src/uar/memory/service.rs` unconditionally imports optional `surrealdb` and cfg-gated `persistence::providers::surreal::SurrealEndpoint`. The struct owns concrete SDK handles, realtime stream types, and adapter lifecycle state in every feature profile. `scoped_storage.rs:93` downcasts directly to `surrealdb::types::Error`.

`MemoryService` itself cannot simply be cfg removed: lib.rs AppState, embedded.rs builder/runtime, tools/memory.rs, admin/memory.rs, auto_capture, background, context_builder, owned memory and controls retain that facade. embedded.rs:469 requires borrowed_scope; embedded.rs:476 requires close_scope. These host-facing type and method contracts must remain available.

The only existing service constructors build concrete SurrealStorage, either owned or host-borrowed SDK client. No generic host MemoryStorage constructor exists. Existing controls.rs already provides explicit unsupported errors for every durable-control operation without surreal-backend. Introducing another backend, generic storage constructor, consent policy fallback, fake memory service, or persistence-to-memory adapter exceeds this bounded repair.

## Minimum proposed implementation

Owned production files: `src/uar/memory/service.rs` and `src/uar/memory/scoped_storage.rs` only. No manifest, lockfile, vendored package, persistence, or kernel edits are needed for the direct unresolved references.

1. In service.rs, gate concrete DB imports (`surrealdb`, SurrealStorage, surreal storage configuration types, SurrealEndpoint) with `#[cfg(feature = "surreal-backend")]`. Likewise gate std/futures imports used only by concrete subscription handling, while retaining any used by common close handling.
2. Gate concrete fields `surreal_storage` and `owned_client`. Preserve common facade storage/config/embedding/lifecycle/owner state and public MemoryStorageOwnership/MemoryCloseReport types.
3. Gate the complete MemorySubscriptionClient, MemoryLiveStream and PendingMemoryLiveClose definitions and every associated impl, including Debug, Stream and Drop. Their types expose the SDK and only realtime/surreal_bus.rs consumes them; that module is already cfg surreal-backend.
4. Gate the existing backend versions of `new`, `initialize_owned_retained`, `build_owned_service`, `from_host_connection`, `initialize_borrowed_retained`, `build_borrowed_service`, `borrowed_scope`, `reconnect`, `transport_generation`, `memory_db`, `raw_surreal_storage`, `subscription_client`, `close_scope`, and free helpers `shutdown_failed_owned_memory_client`, `close_undelivered_memory_service`, `close_memory_resources`.
5. Add same-signature no-backend variants only for methods needed by common API consumers: `new(config: MemoryConfig) -> Result<Self>`, `borrowed_scope(&self) -> Result<Self>`, and `close_scope(&self) -> MemoryCloseReport`. `new` and borrowed_scope explicitly refuse with a stable message requiring surreal-backend. They must not create a DB, start embeddings, infer a host store, enable HTTP, or allocate an inert facade. `close_scope` returns a failure report with scope_released=false and engine_close_acknowledged=false rather than claiming physical engine closure; no valid facade can currently be constructed in this configuration, but the callable contract remains required by the embedded finalizer. Backend close logic remains byte-for-byte behaviorally intact. If the implementer instead factors the existing close implementation to keep shared scope drainage, it must preserve unresolved effects and cannot report an engine acknowledgment without the SDK receipt.
6. In scoped_storage.rs, cfg the existing typed transport_outcome_unknown implementation to surreal-backend; add a no-backend signature returning false (the caller's original error still propagates). This is valid only because this change introduces no usable no-backend service/storage constructor. Do not generalize this false result into an assertion about a future host backend; that future adapter must supply its own outcome classification before it can become usable.
7. Update the service module/constructor docs locally to state concrete backend availability. Keep host MemoryService attachment APIs and owner scope enforcement intact.

No new retries, timeouts, fallbacks, dependency features, or permissive consent behavior. Unsupported branches trace directly to the reported feature-profile failure and absent concrete backend.

## Adjacent coordination required

`src/server.rs:1535` currently uses surreal_bus and subscription_client in a block without cfg surreal-backend. Server-only without backend consequently needs that entire memory realtime/reconnect block cfg gated by its owner. Existing server memory construction can retain new(config) and receive the explicit unsupported error. Do not force `server` to enable surreal-backend to avoid this source boundary.

`embedded-mobile = ["host-persistence"]` and `host-persistence = []` must stay unchanged: neither may select surreal-backend or server. This plan does not change Cargo feature selection.

Uncomfortable limitation: Cargo.toml's nonoptional surreal-memory dependency itself has unconditional surrealdb and surrealdb-types dependencies in its checked-in Cargo.toml. Therefore the proposed cfg repair removes UAR's direct optional-SDK references; it does NOT establish zero transitive database code/dependencies, binary size, compilation success, or runtime acceptance. Eliminating that transitive dependency would require upstream feature separation and a separately scoped change. Host-persistence currently denotes the runtime PersistenceLayer boundary, not a generic MemoryStorage capability.

## Inspection receipts and limits

Only source commands ran: cat/sed/rg against current-waypoint.json, root AGENTS.md, Cargo.toml, versions.toml, gotchas.md, Rust rule/skills, service/scoped_storage/controls/mod/migration and consumers, embedded/server/realtime/persistence provider module boundaries, and the checked-in surreal-memory storage trait/manifest. Two guessed source search paths did not exist (`src/uar/providers/mod.rs`, `src/uar/runtime/embedded*`/`src/uar/server*`); corrected to verified source locations. No verification command ran. Rust workspace/router/best-practices guidance was read; no Cargo was invoked.

Written now: this handoff only. Production implementation, compilation, closure behavior, no-backend refusal and every profile remain unverified. No unrequested production additions. Lead must reconcile final source inventory and obtain exact operator gate authorization after the whole phase is implemented.

## Implementation handoff — repair009 authorized followup

The lead accepted this plan and granted two-file ownership. Authorized implementation is now complete; ownership is released after this handoff. Verification gate remains CLOSED. This is source implementation evidence only.

### File-by-file delivery

- `src/uar/memory/service.rs`: cfg gates concrete Surreal imports; concrete adapter/client/embedding/close-state fields; live subscription client, stream and retained close types/impls; concrete acquisition, borrowed-client, reconnect, raw-client, subscription and close methods; and SDK close helpers. Existing full-surreal method bodies and lifecycle semantics remain intact. `MemoryService`, common memory actions, ownership/report types, host-facing new/borrowed_scope/close_scope signatures remain available. No-backend new and borrowed_scope explicitly refuse the unsupported backend scenario; close_scope returns scope_released=false, engine_close_acknowledged=false and a named unsupported error. No no-backend instance is synthesized. Concrete embeddings field/import is backend-only because its only readers are backend construction/borrowed_scope and controls_store (already gated).
- `src/uar/memory/scoped_storage.rs`: retains typed Surreal connection classification only under surreal-backend. **Plan correction:** the no-backend classifier returns true, not the proposed false. Without a concrete backend classifier, every failed mutating operation has an unknown outcome, so the existing completion path retains its unresolved obligation while propagating the original error. This avoids interpreting an unclassified failure as acknowledged. This branch does not assert a transport cause or retry work. No usable no-backend MemoryService/storage constructor was introduced.
- This handoff and `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-repair-009-memory.patch` record the authorized source delta. The patch contains the complete unified diff against captured before bytes, so reverse application reconstructs the exact pre-edit sources. Temporary before bytes also remain at `/tmp/uar-repair009-memory-service.before.rs` and `/tmp/uar-repair009-memory-scoped-storage.before.rs`.

### Source identity receipts

Python hashlib SHA-256 over the exact file bytes, performed solely to materialize the specifically requested before/after source identity and complete patch; no code import, compile, runtime, dependency resolution, or validation was performed.

| File | Before SHA-256 | After SHA-256 |
| --- | --- | --- |
| src/uar/memory/service.rs | 9a851c3035fd84c2db6e77bfc2bdbefac32dbfcffe197ed60821e4e18201e555 | 85e9b5e00f9b61a0929f020259855ec9f993f508424cc14b8b180d18cb83835f |
| src/uar/memory/scoped_storage.rs | d9bb191ef9712adcddaf04f59c32640c3f259759afda8927d5b3c4b13ff420d5 | 3c4da46c0032265473ada3233521e7150caeb29334655dfa75f261f1f3b6dbbd |

The source identity authoring command emitted exactly those before/after hashes and the patch path. Source-only cat/sed/rg inspection followed the edits. No test/check/lint/format/typecheck/Cargo/build/probe/review gate/service/provider command ran; no tier passed. No git diff --check ran.

### Guards, risks and remaining work

All new cfg/refusal branches trace to the reported optional-SDK feature compilation failure and the real absent-backend boundary. The unknown-outcome branch preserves existing mutation accountability for an unclassified failure. No unrequested production changes, feature widening, dependency updates, network behavior, or permissive fallback were added.

The lead owns the server memory realtime cfg boundary and confirmed it will gate that concrete block separately. No additional common callsite changes were identified by source inspection. Compilation of every affected profile, warnings, runtime memory owner enforcement, constructor refusals, physical SDK closure, and acceptance remain unverified under CLOSED. The nonoptional surreal-memory dependency still includes SurrealDB transitively; this repair does not establish a database-free dependency graph. No generic host MemoryStorage adapter or usable non-Surreal memory backend is provided. These limitations are intentionally retained rather than silently broadening the repair.
