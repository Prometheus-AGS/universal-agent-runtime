# Caller tools runtime handoff — 2026-10-22

Phase: Execute; authoritative active phase remains function-first-release-certification. Verification CLOSED. No tests, builds, lint, formatting, Cargo commands, policy gates, or diff checks were run by this writer. Commands were source inspection or implementation writes only.

## Source ownership and edits

- `src/uar/runtime/turn/client_tools.rs` (new): protocol-neutral caller declaration, controls, canonical provider history, separate caller-visible history, trusted graph and budget snapshots; schema/name validation and ownership lookup.
- `src/uar/runtime/turn/mod.rs`, `request.rs`: expose module and RunExecutionRequest.client_tools default None.
- `src/uar/runtime/turn/resolved.rs`: frozen caller definitions on resolved turn; merge schemas after host collision validation; retain controls, suppress host advertisement for required/named caller choice and all tools for none.
- `src/llm/orchestrator.rs`: populate typed provider controls; accumulate complete-only provider calls; suppress provisional provider caller IDs; validate chosen caller arguments and parallel preference; retain host tool admission/execution; issue stable locator IDs; persist full canonical history and caller-visible predecessor with graph/budget snapshots before ClientToolsRequested; return without further provider request or retained task.
- `src/uar/runtime/manager.rs`: consume prepared full history, prevent duplicate synthetic input, preserve caller instruction content as subordinate user content; restore trusted graph/budget state under fresh current policy; attach budget capture; project pending caller event and context; execute_request_observed returns startup history plus live receiver atomically under history publication lock, failing explicitly if startup history was truncated.
- `src/uar/runtime/graph/turn.rs`, `tools.rs`, `engine.rs`, `nodes/llm_node.rs`: propagate caller handoff through actual graph model host; persist trusted current graph node/state; stop graph at handoff; resume same model node rather than repeating earlier graph effects.

## Shared contracts

`ClientToolInput`: tools Vec<Value>, tool_choice Option<llm::ToolChoice>, parallel_tool_calls Option<bool>, history Vec<Value>, visible_history Option<Vec<Value>>, protocol String; crate-private graph_state Option<Value>, budget_snapshot Option<cost_budget::ClientToolBudgetSnapshot>. Adapter fresh literals use None for all three snapshot/history extras.

Call IDs: `uar_client_<canonical UUID batch>_<canonical UUID call>`.
Base event: ClientToolsRequested {batch_id,calls}; domain adds run_id. Followed by Done and manager resource cleanup/terminal events.

Persistence writer owns prepare/complete and immutable row schema. Continue adapters must set request.session_id from claim.session_id before restoring economic scope. Pending owner key uses verified ActorOwner.session_owner_key(), not subject alone. Separate visible_history prevents hidden host transcript exposure on subsequent handoffs.

Accounting writer owns ClientToolBudgetSnapshot serialization and ModelCallBudget snapshot_client_handoff / restore_client_handoff. Runtime capture/restore calls are wired against agreed APIs; source integration remains unverified.

## Remaining obligations

No compilation, independent review, product acceptance, or release acceptance is claimed. Root must reconcile all adapter literals, wrapper propagation, exhaustive event handling, and final source integration before recording whole-phase implementation complete. Dedicated acceptance writer owns five-interface production HTTP scenarios. Accounting and persistence writers must finish corresponding snapshot implementation. No 1.2.0 image or tag was modified.

## Source integration repair — persisted/public equality

Parent identified two concrete continuation failures: raw caller arguments could differ from host-secret-projected public arguments, and visible predecessor content omitted earlier host-tool-iteration text. Orchestrator now projects caller arguments before caller-call cloning, schema validation, canonical/visible serialization, persistence and event publication. Invalid arguments after projection fail explicitly rather than publishing an unusable schema result. Canonical per-iteration assistant text is scrubbed before serialization; the visible assistant record uses scrubbed cumulative text from every provider iteration in this request. Original caller-visible predecessor history remains distinct from hidden canonical history. No verification commands were run; equality remains acceptance-unverified under CLOSED.

## Output-cap integration

Anthropic SDK `max_tokens` now has host request plumbing: RunExecutionRequest.output_token_cap defaults None; manager intersects that cap with the trusted continuation's retained ClientToolInput.output_token_cap. Primary and shadow resolved turns retain it. Every orchestrator base LlmRequest receives the cap; destination preparation and frozen step assembly lower the real RequestBudgetContract output limit with i64::from(cap), never raise it. Graph model calls share this path. Pending continuation retains the cap; persistence prepare must min-merge later request caps so continuation cannot raise the original ceiling. Provider writer owns native/Liter final wire enforcement. No checks executed.

## Cleanup settlement publication repair

Parent observed adapters could publish successful handoff before trusted host resource cleanup. Shared EventHistory now holds ClientToolsRequested privately instead of assigning an event ID, broadcasting, persisting public history, or informing completion capture on arrival. Error/Cancelled clears this staged handoff permanently. A successful final RunDone/RunDoneWithUsage flushes it immediately before the terminal event, after ordinary/graph host cleanup paths. Root still owns adapter awaiting of actual producer settlement after terminal and admission checks against the source run before accepting continuation; terminal publication alone does not join the producer handle. No verification commands executed.

## Actual producer join helpers

RunManager now exposes crate-private settled_run_status(run_id) and await_settled_run_status(run_id). The latter borrows retained ordinary handles across await or joins ActorRootBinding, then joins GraphRootSupervisor::join_run including session cleanup. It never uses timer polling. Join failure marks the active run Error. Existing authenticated context status helper delegates after checking identity. Added thread/graph_host.rs ownership for join_run. Important restart limitation communicated to lead: get_run is active-only; a durable source-settled marker must be acknowledged after real join and before public handoff so restart can distinguish a published settled batch from a pending row created before failed cleanup. No verification run.

## v1.2.2 native ingestion startup repair

Observed gate001 candidate failure: native startup used hardcoded /data/ingest, and directory creation failed inside the admitted watcher before readiness, masking the I/O cause with a dropped acknowledgment. Bounded source repair changes only src/config.rs and the ingestion startup span of src/server.rs: ServerConfig.ingest_directory is a PathBuf accepting a YAML string, default /data/ingest for existing container behavior. Startup creates the configured directory before registering/admitting the watcher and reports the exact configured path and underlying I/O error. Existing watcher readiness, cancellation, and join remain intact. Supervisor writer received the key for an evidence-owned local directory. Source search identified one required application ServerConfig literal outside ownership, tests/settings_persistence.rs:136; lead notified. No tests, builds, checks, probes, formatting, Cargo/docs/supervisor edits, or inventory edits performed by this writer. Frozen 1.2.0 and published 1.2.1 untouched; repair targets 1.2.2. Verification remains lead-owned and unperformed by this writer.
