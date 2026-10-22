# OpenAI adapter implementation handoff

Phase: function-first-release-certification. Verification lock CLOSED.

Owned source: `src/uar/api/openai/types.rs`, `src/uar/api/openai/routes.rs`.

Implemented request function declarations including `strict`, `tool_choice` translation (auto/none/required/named), `parallel_tool_calls`, full ordered text history, assistant nullable content with tool calls, and tool messages with tool_call_id. Canonical history uses the existing llm Message format; assistant null content becomes empty internal text. System/developer map to system.

Implemented shared ClientToolInput and durable prepare/Continue/Replay integration. Adapter validates current authenticated scope and agent before preparing. Unsupported or absent durable persistence rejects caller-tool requests before execution. Claimed continuations persist the exact JSON completion plus SSE event projection before public terminal success.

`stream:false` returns a JSON chat.completion and provider usage when supplied. `stream:true` emits text live, captures actual chunks, projects only the committed ClientToolsRequested batch as indexed function-call deltas, then tool_calls finish_reason and DONE. Caller arguments become available after durable batch admission; raw host tool events are never exposed as caller requests. Errors, cancellation, lagged/closed observation, or failed completion persistence emit errors without successful stop.

Unresolved integration: initial run startup still uses execute_request then subscribe, pending lead's atomic observed API; a fast run can win this race. Runtime writer has been notified to retain system history and store visible assistant-empty canonical text consistently for continuation equality. Shared persistence writer has been notified that suffix-only result recognition needs structural history handling when a new user message follows unresolved caller results.

No tests, checks, builds, formatter, compiler, probes, or review gates ran. Source inspection commands only (`cat`, `sed`, `rg`, `ls`, `git diff`) plus authorized source/document writing. No test acceptance or compilation claim. All behavioral claims above describe authored source and remain unverified until the operator opens the gate.

No unrelated guards added. Validation guards implement the caller-input trust boundary; storage support guards enforce durable continuation availability; observation-loss and terminal persistence guards prevent false success.

Update: durable prepare now uses `UserContext::session_owner_key()` plus verified tenant, avoiding bare-subject collisions across credentials. Fresh ClientToolInput initializes visible_history=None; shared prepare replaces it on continuation so runtime retains private canonical host history separately from caller-visible history. Runtime writer reports caller system text is retained in order as subordinate provider user context. Persistence writer reports historical exchanges now require exact completed scoped receipts; appended user messages cannot bypass unresolved result proof. Those are shared-source handoffs, not independently verified behavior.

Final integration update: OpenAI now calls `execute_request_observed(request)` with the runtime writer's exact announced result `(run_id, Vec<StreamEvent>, broadcast receiver)`. Both JSON and SSE paths consume captured history before live receiver events; completion IDs derive from the run ID. Runtime helper implementation is owned by the runtime writer and was still in progress at this update. Removed duplicate host_history conversion; manager consumes ClientToolInput.history. Verification remains CLOSED and no gates ran.

## Cohort 11.1 terminal settlement

Both JSON and live SSE now retain ClientToolsRequested privately until a final
RunDone/RunDoneWithUsage arrives. They then await the exact producer/graph-root
settlement through manager.await_settled_run_status(run_id), accepting only
RunStatus::Done. Error, cancellation, unconfirmed settlement or a join failure
returns/emits an error without caller requests, successful finish, or completed
continuation receipt. Live text remains live; captured startup history is still
consumed before the receiver. Tool chunks and completion persistence occur only
after the Done settlement acknowledgment, and terminal provider usage is retained.

The shared prepare(manager,store,...) signature and source-Pending settlement
admission added by the lead are preserved. Only routes.rs and this handoff changed
in this cohort. No verification command or review gate ran; CLOSED remains in force.

## Cohort 11.1 durable source-settled acknowledgment

After final Done settlement, both JSON and SSE caller-handoff paths invoke the
shared confirm_handoff_for_run(manager,store,full owner key,tenant,agent,run) helper
before constructing/publishing caller chunks or completing a predecessor receipt.
Its failure returns/emits client_tool_handoff_unconfirmed and no successful
handoff. This persists the trusted source_settled acknowledgment needed for cold
Pending continuation; a historical RunDone event alone cannot authorize restart.
The SSE path owns manager, owner, tenant and agent data across its async lifetime.
Only routes.rs and this handoff changed. Verification remains CLOSED; no check ran.
