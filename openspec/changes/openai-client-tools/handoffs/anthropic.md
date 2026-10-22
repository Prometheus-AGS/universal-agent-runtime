# Anthropic Messages caller-tools handoff

Execute phase, verification CLOSED. All activity was source inspection or source/documentation implementation. No check, test, format, compiler probe, build, review gate, commit or push ran.

Owned source: src/server.rs Anthropic Messages request/types, conversion helpers, api_messages and new api_bound_anthropic_messages projection helper. Follow-up ACP initializer/session restore adjustments remain inside previously owned ACP files.

- Native tool_choice auto/none/any/tool becomes shared Auto/None/Required/Named. Native disable_parallel_tool_use is inverted into parallel_tool_calls; contradictory extension/native parallel controls fail explicitly. Unknown native choice fields fail deserialization.
- Native strict and description are preserved when converting function declarations. Empty tool names no longer disappear through filtering; bound runtime validates declarations. Unsupported tool-definition members fail explicitly.
- ensure_toolu_id preserves host-issued uar_client_ locators across native tool_use and tool_result conversion. Other legacy native ID behavior remains.
- Bound registered agents now execute the shared RunManager runtime rather than prepend an agent prompt and bypass host tool execution. Selected model/provider is restriction-only host_policy_constraint, so registered artifact policy stays intact. Prompt-cache override is in that same turn constraint. Existing authorized_http_model checks still happen before dispatch; runtime performs its own host credential/policy admission.
- Shared client_tools.prepare validates owner/tenant/agent/protocol/schema/history before claim. Native continuation checks durable session binding first. Continue uses the saved session_id, restored graph state, and shared budget capture through the runtime input.
- RunManager.execute_request_observed supplies atomic startup history plus live domain events. Text is projected live into native SSE. Host-tool lifecycle remains internal. Persisted ClientToolsRequested becomes native tool_use blocks with host-issued IDs. Terminal RunDone/RunDoneWithUsage maps to native end_turn/tool_use; errors yield native error events and no success marker.
- Native terminal response and native SSE projection are committed with client_tools.complete before message_delta/message_stop or JSON success. Matching duplicates replay the stored JSON/native events without execution. Failed or interrupted claims stay claimed per the shared fail-closed contract.
- Fresh host-only ClientToolInput fields visible_history/graph_state/budget_snapshot are None at ingress. Shared service/runtime own restoration.
- Unbound direct forwarding now passes explicit typed tool controls and preserves strict; typed provider capability errors are exposed as invalid requests instead of generic upstream failures.

Dependencies: shared persistence service implements Anthropic-specific lossless visible-history comparison for native roundtrip differences (JSON argument whitespace, empty assistant content and null/absent tool_calls). Runtime manager's new observed entry point is required. No new exports or dependencies were introduced.

Uncomfortable limits: bound requests reject otherwise-unhandled top-level extension fields explicitly (including generation controls formerly silently ignored), rather than claiming to honor unsupported settings. Stream receiver lag or loss is an explicit error; it never fabricates a completed result. Claims interrupted after side effects remain unavailable until operator recovery. Actual compilation, credential readmission, provider behavior, durable retry/restart, and native JSON/SSE acceptance remain unverified under the operator lock.

## SDK max_tokens integration correction

The native SDK requires max_tokens. It is now a typed positive optional u32 request member instead of an unsupported flattened extra. Bound execution sets host-only RunExecutionRequest.output_token_cap; the runtime writer owns propagation through the shared planner. Unbound forwarding sets LlmRequest.output_token_cap. The native Anthropic driver intersects it with its existing authorized/default maximum. Liter intersects an existing final-wire budget output reservation and the caller cap and serializes the smaller limit. Anthropic reasoning settings that would increase the wire ceiling beyond the cap fail explicitly. No cap widens an existing authorized output reservation. The unreachable former bound-agent prompt-prepend branch was removed. Remaining unsupported bound generation settings still fail explicitly. This supersedes the max_tokens limitation above.

Shared new request field LlmRequest.output_token_cap: Option<u32> requires None in every other initializer; lead notified. Native/Liter-owned request literals are updated. Runtime-owned RunExecutionRequest/output preparation updates are coordinated with the runtime writer. No verification ran for this correction.

## Native SDK session continuity correction

A native SDK caller does not need to send UAR session_id. On a result submission the host-issued call locator is looked up under the authenticated owner/tenant/agent; the stored protocol must be anthropic. An explicitly supplied session_id must match the stored session. An omitted session_id is accepted and Continue restores claim.session_id internally for graph/budget continuity. The caller does not invent or persist a private UAR session field. Fresh ClientToolInput.output_token_cap now captures req.max_tokens before preparation so the persistence/runtime min intersection survives later continuations. No verification ran.

## Cohort 11.1 terminal settlement

Bound Anthropic execution retains the observed run_id and awaits RunManager.await_settled_run_status after the terminal event. Only settled Done can complete a claim or publish native terminal success. Error, Cancelled and join/settlement failure yield a native error. Caller-tool content blocks are staged until successful terminal settlement; readable text continues live. ACP buffered caller-tool responses likewise await settled Done before projecting pending state, completing a claim or returning the response. No verification ran.

Cohort 11.1 durable cleanup acknowledgment: before publishing staged native caller tool_use blocks or a pending-handoff success, bound Anthropic calls shared confirm_handoff_for_run with the same authenticated owner/tenant/agent/run scope. The service records monotonic source_settled acknowledgment after actual settled Done. Later continuation preparation can admit that durable acknowledgment after restart without pretending a history terminal proved cleanup. No verification ran.
