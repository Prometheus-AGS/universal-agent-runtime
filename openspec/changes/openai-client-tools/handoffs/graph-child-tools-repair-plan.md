# Actual graph child caller-tools repair plan

Status: planned source repair; no production edits or verification. Gate010 proves the ordinary five-interface paths but does not certify the installed graph child path.

## Production path and observed blocker

Desktop server and embedded startup install defaults::orchestrator_graph through RunManager::with_agent_graph. The graph is RouterNode followed by AgentNode(general-purpose/rust-reviewer). The server has no discovered YAML/API graph-definition loader. LlmNode exists as a real embedding API building block, but replacing the installed graph solely for this test would not fix the requested production path.

The router completed in candidate-2.log. The next AgentNode operation uses GraphThreadDelegate's governed spawn_agent admission, whose descriptor requires approval. Gate010's driver never resolves it. Preserve that governance path and add actual approval interaction to the acceptance driver.

## Minimal host contract

Keep AgentSpawnRequest as model-facing intent. Do not add authority fields to its JSON or fork caller tool metadata through prompts. Add a separate host-only GraphCallerTurnCapture passed by a dedicated graph spawn method alongside AgentSpawnRequest. Its construction requires the root's existing verified identity and current graph context. Capture contains normalized ClientToolInput, root API agent/run/session identities, and the current parent GraphState with the AgentNode resume cursor. It is not Deserialize and cannot be set by model arguments or API forwardedProps.

Propagate that capture through GraphThreadDelegate -> AgentToolContext host-only graph-spawn method -> ThreadService's local child entry -> HostedThreadTurn -> GovernedThreadExecutor's RunExecutionRequest. Reuse all existing spawn admission, parent/child policy intersection, model/credential bindings, approvals, ownership, cancellation and shared root budgets. Ordinary model-driven spawn and remote delegation keep their existing paths; no automatic propagation to arbitrary grandchildren.

The child receives caller function schemas/controls through request.client_tools. On continuation its prepared canonical worker history must outrank generic inherited-history seeding; do not strip paired tool traffic through HistoryForkMode. The child still builds its own current authoritative artifact/system policy. The root graph's data and route remain in the separate trusted graph state.

## Durable handoff and return

A delegated caller batch is correlated to the root API agent/run/session, not the worker's public agent ID. Persist a host-verified executing-child run/thread identity alongside it so the graph delegate can prove which child produced it. Root identity comes from the typed host capture, never caller JSON. Capture the existing root-shared ModelCallBudget snapshot; never replace it with a fresh child budget.

Child handoff persistence precedes any caller event. The child finishes its real producer cleanup and durable first-turn record through the existing completion capture. Avoid changing every AgentThreadResult variant solely for graph transport: GraphThreadDelegate can return a GraphChildOutcome wrapper containing the existing terminal AgentTurnOutcome plus an optional validated durable caller handoff. Obtain that optional handoff only by exact owner/root/executing-child durable lookup after wait_first_turn and actual child settlement. A generic empty Completed output without such evidence remains an error.

AgentNode handles the wrapper's handoff explicitly: forward the exact saved caller-visible assistant text and pending calls through the parent graph's trusted event sink; set _client_tool_handoff; finish this graph segment. Do not fabricate tool results or expose hidden child host-tool results. The parent's existing staged event publication and API confirmation must still await all parent/child producer/resource joins before public success.

Manager graph finalization must skip its normal _route/_agent_output attribution and delegation_output_missing error when _client_tool_handoff is true. It must not add a `[general-purpose]` text prefix to a caller-visible handoff whose persisted transcript has different bytes. Run status is successful only if actual graph, child, sandbox, terminal, MCP and outer producer cleanup succeed.

## Continuation and crash recovery

The root batch's stored graph state resumes the same AgentNode, preserving the selected worker and skipping RouterNode. The shared prepare CAS validates principal, tenant, root API agent, full visible transcript, schemas/controls and exact results before restoring hidden canonical worker history. Re-enter through a newly admitted root with current policy and credentials. Re-spawn/resume the selected worker through the same governed graph-spawn capability; answer any newly required real approval. Do not bypass approval by treating a previous root's approval as a reusable grant.

The worker's next provider request receives the prepared canonical transcript plus caller results, retaining hidden prior host results without executing them again. Persist the next handoff with the new root and executing-child identities. Identical completed retries replay the existing admitted segment, never create another child. A crash before settlement acknowledgment remains unconfirmed; a crash after acknowledged handoff can resume from the durable root graph state and worker transcript without replaying routing or past host effects.

## Real approval and acceptance driver

Submit graph requests while a companion acceptance task uses authenticated GET /api/uar/runs to find the exact new orchestrator root, then GET /api/uar/runs/{run_id}/tool-approval. Resolve only the observed pending spawn_agent challenge with POST on that same tool-approval route using {"approved":true,"approval_id":"<record.challenge_id>"}. The source DTO ToolApprovalRequest also permits optional resume for cold resource renewal; warm graph approval does not supply it. Do not globally disable governance, change spawn descriptor approval class, or approve an unrelated call. Retain the decision receipt in gate evidence. AG-UI/A2A exposed approval interrupts use their native supported resolution paths where they close a wire segment; correlate the same root/challenge.

Run all five native interfaces against orchestrator-agent with the real builtin RouterNode/AgentNode graph and persisted general-purpose marker. Require actual provider sequence router, worker, worker across handoff/continuation; no router replay. Require client schema/choice in worker requests, exact caller IDs/results and current governed worker admission. The separate core mixed-batch and mixed-crash suites verify real host-effect preservation; this graph suite does not claim the additional graph-plus-mixed-host cross-product. Retain actual child/root IDs and approval receipts. Repeat a cold handoff stop/join/restart/resume and verify the same sequence/canonical result. Include explicit graph cancellation and failed cleanup evidence; do not mark a timeout as a caller handoff.

## Ownership and acceptance boundary

1. Runtime writer: graph/delegation.rs, graph/nodes/agent_node.rs, graph/tools.rs, manager graph-finalization span; root capture and parent event forwarding.
2. Thread writer: thread/control.rs, thread/service.rs, thread/kernel.rs and host-only capture plumbing; preserve model-facing request contract and child governance.
3. Persistence writer: continuation executing-child identity and immutable validation; root/child correlation and common restore data.
4. Acceptance writer: real approval driver, five-interface graph and cold graph cases. Lead owns full-phase inventory, source review and serialized build/gate execution.

No verification is authorized by this plan itself. All production implementation and required acceptance authoring must be complete before the lead opens its approved local gate. Never rebuild or move frozen 1.2.0 or published 1.2.1.

## Runtime writer implementation handoff — task 13.21

Authored only graph/delegation.rs, graph/nodes/agent_node.rs and manager.rs graph construction/finalization spans. GraphThreadDelegate now captures verified root caller data with manager/persistence/event host capabilities. Its local execute accepts the trusted current graph state; AgentNode supplies the same-node resume cursor. Caller-aware local spawning uses AgentToolContext::spawn_graph, while ordinary and remote spawning retain their prior paths.

GraphChildOutcome wraps the actual first-turn outcome and optional validated caller handoff. After real first-turn completion and child producer settlement, the delegate loads the root-scoped pending continuation. It verifies owner, tenant, root API agent/run/session, Pending state, execution_source agent/run/thread against the actual selected child, exact pending call IDs, assistant role and public text shape. It never confirms the still-executing parent. AgentNode publishes saved visible text and caller calls through the existing parent host event sink, sets _client_tool_handoff and returns Finished. Manager skips normal attributed route text and delegation_output_missing when that flag is set; existing cleanup/error handling and staged terminal publication still apply.

Lead owns GraphCallerTurnCapture/delegation definitions, persistence execution_source, orchestrator root-scope writes, and API acknowledgment. Thread writer owns spawn_graph and HostedThreadTurn propagation. Acceptance writer owns real approval resolution. No changes to GraphContext or generic AgentThreadResult were needed. No tests, builds, checks, imports, probes, formatting, network or runtime commands executed by this writer. Whole-cohort source integration and local functional acceptance remain pending.
