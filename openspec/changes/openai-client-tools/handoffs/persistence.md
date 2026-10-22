# Durable caller-tool persistence handoff — 2026-10-22

Phase: Execute. Operator verification gate: CLOSED. No test, check, build, formatter, compiler, review gate, or acceptance script was run. Commands used only source inspection and implementation/document writing. Others share this checkout; preserve their edits.

## Implemented source

- `src/uar/persistence/client_tools.rs`: owner/optional tenant/agent/batch-bound continuation record; immutable protocol, full canonical host transcript, exact visible transcript, client schemas and pending-call IDs. State moves only Pending → Claimed → Completed. Atomic claims retain the full call-ID-to-result map; incomplete, extra, duplicate pending IDs and result replacement are rejected. Completed records retain both response and event projection and cannot transition further.
- `src/uar/persistence/mod.rs`: additive fail-closed capability, create, scoped load and compare-and-swap methods. No in-memory fallback.
- `src/uar/persistence/providers/client_tools/postgres.rs` and provider registration: insert-if-absent, exact scoped load and JSONB compare-and-swap update.
- `src/uar/persistence/providers/client_tools/surreal.rs` and provider registration: durable-backend gate, transactional insert-if-absent, exact scoped load, compare-and-swap update and typed transaction-conflict handling following existing AG-UI storage convention.
- `migrations/20261022010000_client_tool_continuations.sql`: durable PostgreSQL table, owner/agent index. Loaded by existing SQLx migration discovery.
- `migrations/surrealdb/client_tool_continuations.surql` plus `src/uar/persistence/providers/surreal_host.rs`: durable schema and startup registration.

## Integration contract

`ClientToolContinuation` fields: owner_key String, tenant_id Option<String>, agent_id String, batch_id String, protocol String, canonical_transcript Value, visible_transcript Value, client_schemas Value, pending_call_ids Vec<String>, state ClientToolContinuationState, result_batch Option<BTreeMap<String,Value>>, completed_response Option<Value>, completed_events Vec<Value>.

Trait methods: supports_durable_client_tools(); create_client_tool_continuation(&row) -> Result<bool>; load_client_tool_continuation(owner,tenant,agent,batch) -> Result<Option<Row>>; compare_and_swap_client_tool_continuation(before,after) -> Result<bool>.

Runtime must persist Pending before publishing handoff; perform current authorization and visible-binding/schema validation before claim; build a Claimed copy with the complete result map and win CAS before any continuation effect. CAS loss requires reload: only identical Completed result batches may replay saved response/events; Claimed rejects in-flight or interrupted retries. A cold Claimed row must never be reacquired or reexecute host work. Completion must be persisted before publishing final response/events. Store validation preserves transcript/schema identity, but semantic validation of their protocol-specific contents belongs to the shared runtime/adapters.

## Guard rationale and limits

All guards protect the explicit caller-result trust boundary or the named duplicate/concurrent/cross-owner continuation scenarios. No unrelated feature or guard was added. No new dependency or version change. No credentials changed, external action taken, commit or push performed.

Unverified: compilation, migration application, database concurrency, restart/replay behavior and protocol integration. Verification remains prohibited until lead reconciles the entire phase inventory and operator explicitly authorizes exact commands. Source inspection established existing provider patterns and migration discovery only; it is not acceptance evidence.

## Adapter integration addendum

Parent approved immutable source_run_id String and session_id Option<String> additions; runtime confirmed supplying both. Added load_client_tool_continuation_for_run(owner,tenant,agent,run), provider implementations and run lookup indexes. Canonical and visible transcript arrays and schema binding object now validate at storage boundary. Shared ingress service details are in services.md. Verification remains CLOSED; no checks ran.

## Protected runtime continuity addendum

Added immutable graph_state Option<Value> and typed budget_snapshot Option<cost_budget::ClientToolBudgetSnapshot> at runtime/accounting writers' request. Added ScopedPersistence retained dispatch forwarding after parent identified production facade defaults would reject the feature. Exact graph and budget implementation contracts remain owned by those writers. No verification ran.

## Native output restriction

Added immutable output_token_cap Option<u32> to retain native Anthropic output restrictions; storage rejects zero. Shared prepare takes the minimum requested/saved cap. No migration shape change because field lives in protected JSON payload. Gate CLOSED; no verification ran.

## Physical cleanup acknowledgment

Added serde-default source_settled bool, initially false. A Pending false→Pending true transition may change only this flag; Pending→Claimed preserves true and Claimed/Completed require true. Shared confirmation helper owns actual joined-source acknowledgment before public delivery. Durable acknowledged Pending may continue after restart without a live source, while unacknowledged rows fail closed. No automatic Claimed recovery. No verification ran.
