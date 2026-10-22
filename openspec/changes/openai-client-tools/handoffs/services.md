# Shared caller-tool ingress service handoff — 2026-10-22

Phase Execute. Verification gate CLOSED. All commands in this packet were source inspection or authorized implementation/document writing. No test, lint, format, typecheck, compiler, build, review gate or acceptance command was executed. Preserve other writers' checkout edits.

## Owned implementation

`src/uar/api/client_tools.rs` exports:

- `prepare(store: &dyn PersistenceLayer, owner: &str, tenant: Option<&str>, agent: &str, input: &ClientToolInput) -> anyhow::Result<ClientToolPreparation>`.
- `ClientToolPreparation::Fresh(ClientToolInput)`, `Continue { input: ClientToolInput, claim: ClientToolContinuation }`, or `Replay { response: Value, events: Vec<Value> }`.
- `complete(store, &claim, response: Value, events: Vec<Value>) -> anyhow::Result<()>` persists the exact response and event projection before delivery. Identical already-committed completion is accepted; conflicting completion rejects.
- `load_by_call_id(store, owner, tenant, agent, call_id) -> Result<Option<ClientToolContinuation>>` performs exact authenticated scope lookup and requires the call ID belongs to the located pending batch.
- `batch_id_from_call_id(call_id) -> Result<String>` accepts exactly `uar_client_<canonical batch UUID>_<canonical call UUID>`, agreed directly with runtime writer.
- `schema_binding(&ClientToolInput) -> Result<Value>` captures declarations plus tool choice and parallel controls as one immutable object.

Parent owns the `api/mod.rs` export and confirmed adding it. Runtime owns `ClientToolInput`, Pending persistence and caller-call ID generation. Native adapters normalize to common OpenAI-style assistant tool_calls/tool messages.

## Processing contract

Prepare validates the common declaration input. A continuation request ends in one contiguous complete tool-result batch. Each tool message carries tool_call_id and content. IDs must locate exactly one batch; duplicate result IDs reject. Preceding visible history must match the saved caller-visible transcript exactly, schemas/provider controls and protocol must match, and the result ID set must exactly match all pending calls. CAS Pending → Claimed serializes competing submissions before any effects. The returned history uses saved canonical host history, including hidden host results, plus new caller tool results. Identical Completed retries replay saved response/events; Claimed retries reject, including restart after interrupted execution. No automatic retry or reacquisition exists.

For stateful native adapters, load the bound source run/batch first, compare session/task/thread/native-run bindings at current authenticated admission, then reconstruct saved visible history and append normalized results. A call ID only locates evidence; it is not authority. Adapters must re-admit current authorization and retain the claim until complete saves the terminal wire projection. Publishing admission alone is not completion.

## Persistence extension needed by adapters

Added immutable `source_run_id: String`, `session_id: Option<String>` to the shared persisted record, after parent approval and runtime confirmation that both are available. Added `PersistenceLayer::load_client_tool_continuation_for_run(owner,tenant,agent,run)` and scoped PostgreSQL/Surreal implementations/indexes so cold A2A/AG-UI/ACP inspection can project pending state without requiring a client result ID. Ambiguous multiple batches for the exact source run reject rather than choosing one. Runtime must fill the new fields.

Storage additionally requires canonical/visible transcript arrays and client schema binding object. All APIs remain fail-closed on unsupported/non-durable backends. All guards trace to the explicit duplicate, concurrent, cross-owner, malformed-batch or transcript-forgery requirements.

## Remaining unverified

Compilation, protocol integration, migration application, wire projection fidelity, real database concurrency and restart delivery remain unverified because the operator lock is CLOSED. No dependency, credentials, commit, push or external resources changed. Source identity must be collected by lead only within the operator-authorized inventory process; this handoff makes no acceptance claim.

## Source inspection follow-up

OpenAI and runtime writers identified a concrete ingress gap: a caller could append a user message after forged tool results and reach Fresh without a trailing result suffix. The service now structurally validates every caller-visible assistant tool_calls/tool-result exchange against scoped durable evidence, full saved visible prefix, exact IDs and result set. A historical group followed by more messages must already be Completed with identical results; orphan results, host-tool IDs, unresolved groups and mixed batches reject. No check was run.

Runtime added ClientToolInput.visible_history Option<Vec<Value>>. Prepare sets it to the original visible input before replacing history with protected canonical transcript, so subsequent handoffs do not publish previously hidden host results. Adapters initialize visible_history None. This field is trusted in-process context, not a wire input.

## Lifecycle, native roundtrip and runtime-state additions

Parent source inspection found the production ScopedPersistence facade would otherwise inherit unsupported defaults. Added retained dispatch forwarding for capability/create/load/load-for-run/CAS and classified both reads as read-only lifecycle operations. This preserves existing drain/uncertain-effect semantics.

Anthropic adapter source conversion represents function arguments as objects and omits empty assistant text. Anthropic-only visible transcript comparison now normalizes argument JSON string formatting and assistant null/empty/absent text/tool_calls. Nonempty text, function names, IDs, tool results and remaining message structure remain bound. Other protocols retain exact JSON comparison.

Runtime requested protected graph continuation and cumulative accounting capture. Added immutable graph_state Option<Value> and budget_snapshot Option<cost_budget::ClientToolBudgetSnapshot> to durable records; prepare restores both into host-only ClientToolInput fields. Runtime owns real graph cursor restoration and capturing snapshots before handoff; accounting writer owns the typed snapshot/restore implementation. These are source integration dependencies, not accepted behavior. No verification ran.

## Final integration status

Runtime/accounting writers confirmed budget snapshot capture always supplies Some; durable row validation now requires it for create, load, claim and completion. Missing accounting continuity cannot silently restart from zero. The snapshot type is public with private fields to fit the public durable-record field.

Exposed `pub(crate) validate_visible_history(store,owner,tenant,agent,input)` for central post-target admission without reacquiring a claim. It selects protected input.visible_history when present, otherwise caller history. This is evidence lookup only. ScopedPersistence owns lifecycle scope enforcement; protocol adapters still own current user/session/task/thread admission and must supply the full verified principal storage key as owner.

No verification commands were issued. Remaining integration behavior is unverified under CLOSED gate. All extra source changes above were specifically requested by the lead or named observed source problems from adapter/runtime writers.

## Native output restriction integration

Parent/runtime requested native Anthropic max_tokens continuity. Added immutable row.output_token_cap Option<u32>, positive when supplied. Prepare intersects requested and saved cap using the minimum, with None unbounded. Cap is intentionally excluded from immutable schema_binding comparison so a continuation may lower its output limit while never raising its saved limit. Protected graph state, budget and visible transcript restoration remain intact. Runtime owns actual provider cap intersection. No check ran; gate CLOSED.

## Physical cleanup acknowledgment and restart continuation

The lead's source-settlement guard initially required a live source run, which would reject every otherwise valid pending continuation after process restart. Added monotonic durable source_settled acknowledgment. `confirm_handoff_for_run(manager,store,owner,tenant,agent,run_id)` loads the exact scoped batch, awaits actual `RunManager::await_settled_run_status` Done, and CASes only source_settled false→true while Pending. Existing true acknowledgment is idempotent. Every adapter must call it before public caller handoff; a run-history terminal record cannot create the acknowledgment.

Prepare retains the lead's manager-first signature. Pending requires durable acknowledgment; when the source is still live it additionally requires settled Done, while an absent source after restart may rely on the stored acknowledgment. Claimed never reacquires. Store creation requires Pending/source_settled false, only the exact acknowledgment update or existing progression is allowed, and Claimed/Completed require true. Runtime initializes false. All four protocol writers and lead were notified directly. No verification ran; gate CLOSED.
