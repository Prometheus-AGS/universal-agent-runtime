# Production HTTP caller-tools acceptance authoring

Phase Execute. VERIFICATION CLOSED. No Python program was run, imported,
syntax-checked, compiled, formatted, or delegated to a review gate. All work was
source inspection or authorized source/document writing. These are authored
executable acceptance programs; there is no acceptance receipt.

## Owned source

- `tests/fixtures/client_tools_provider.py`: real HTTP provider peer (OpenAI and
  Anthropic JSON/SSE), actual fragmented provider arguments, explicit 2-input /
  3-output token usage, multi-round functions, parallel controls, before/after
  graph-node markers, held upstream responses, invalid/truncated provider streams,
  private synthetic trace, and stateless HTTP MCP at `/mcp`. MCP initialize,
  tools/list, tools/call and ping expose a safe `effect` tool whose actual calls
  increment an independently observable counter. The fixture never supplies a
  fake runtime executor, persistence store, or in-process provider adapter.
- `scripts/release/client_tools_acceptance.py`: native public HTTP drivers for
  OpenAI, Anthropic, AG-UI, official A2A 1.0 JSON-RPC (including
  SendStreamingMessage), and ACP JSON-RPC/stream. Installed candidate, configured
  durable database, selected agents/models, admission and lifecycle remain real
  external collaborators supplied by the operator. HTTP response evidence is
  retained in a mode-0600 report; authorization headers/tokens are not recorded.

OpenAI and Anthropic model inputs are separate: `--openai-model` optionally
selects the OpenAI model/registered-agent target and defaults to the scenario's
agent; `--anthropic-model` is the required provider-qualified Anthropic selection.
This permits one invocation across all five protocols. `--agent-id` chooses the
common configured agent; budget/graph modes select their dedicated real agents.

## Authored matrix

All five interfaces exercise JSON/native and streaming caller loops, earlier
user/assistant/current-user history, two caller batches, declared JSON schemas,
auto/none/required/named choice, parallel false, native result IDs, and completed
identical retries without another provider contact. Strict forwarding is asserted
on the four interfaces whose native declarations express it. AG-UI uses its
native frontend ToolSchema and `forwardedProps.uar.client_tools` choice/parallel
extension. Required/named cases assert the first handoff/forwarded restriction;
they do not demand a final text answer while requiring a function every turn.

Negative batches cover forged locator, cross-owner credential, incomplete batch,
duplicate ID, altered predecessor history, altered declarations and conflicting
completed retry. Each rejected submission must make zero new provider contacts.
The program rejects identical cross-owner credentials at startup.

Simultaneous submissions use two actual HTTP request threads against one durable
pending batch. The peer holds the first resumed provider response while the
second request races the durable claim. At least one completes; the two requests
must cause exactly one provider contact, no repeated host effect, and the later
identical completed retry must replay without contacting the provider.

Disconnect coverage captures the actual native continuation request, sends it on
a real HTTP socket, waits until its provider request is independently observed
held, closes the candidate socket, and releases the upstream peer. Retry may
replay a persisted completion or reject a retained claim; it may never reexecute
the provider or real MCP host effect. This explicitly allows fail-closed claimed
recovery instead of asserting a completion that disconnect did not establish.

Invalid/truncated upstream streams go through actual provider HTTP. Every public
interface must report failure. OpenAI successful stop/tool_calls terminals,
Anthropic end_turn/tool_use stop reasons, and successful AG-UI terminals are
forbidden on this path. These cases exercise lost/incomplete provider observation
without mutating private runtime event channels.

Mixed host/caller execution uses an operator-configured actual HTTP MCP registry
entry against this peer's `/mcp`. `--host-tool-json` supplies its *real advertised
UAR tool name*, including the registry namespace, and valid arguments. Only
caller-owned functions may become public requests. The peer's actual MCP effect
counter must equal one after completion and retry. Concurrent/disconnect cases
also retain that effect count when this collaborator is configured.

## Cold persistence, graph cursor and budgets

`restart-seed` / `restart-resume` save each protocol's exact pending history and
native task/thread/session identity, then submit after an operator-controlled
real stop/join/restart against the same durable database. A distinct
`--instance-identity` is required. Identity is supplied evidence, not remote
process attestation; retain the supervisor lifecycle receipt. The provider/MCP
peer stays running so its traces/counters span candidate restarts.

`graph` and `graph-restart-seed` / `graph-restart-resume` require
`--graph-agent-id`, `--graph-before-marker`, `--graph-after-marker`. Configure an
actual two-model-node graph with those distinct node prompt markers. Actual
provider request sequence must be before/before/after: initial first-node caller
handoff, continuation at that same node, then one next-node request. Skipped or
replayed graph nodes fail. Cold graph mode asserts this after actual restart.

`budget` and `budget-restart-seed` / `budget-restart-resume` require
`--budget-agent-id` with a real cumulative token ceiling below 10 while admitting
the initial 5-token turn. Resumed work must fail specifically at the budget
boundary. Cold budget mode uses the saved pending batch after actual restart;
an identical failed-claim retry must make no further provider contact. This
proves accounting retention through the public path rather than resetting a
private counter or injecting a budget mock.

## External gate inputs and honest limits

Required inputs: installed candidate URL, configured provider-peer URL, two
credential environment-variable names with genuinely different scoped principals,
common agent ID, Anthropic provider-qualified model, and report path. Optional
existing-deployment inputs are workspace ID, A2A route and metadata JSON, ACP path,
and OpenAI selector. The peer requires port-file and trace paths and defaults to
loopback. The operator must configure the actual candidate provider and HTTP MCP
entries, real durable DB, graph agent/node prompts and budget agent policy. The
fixture never invents a registry alias or lifecycle execution command.

Matrix reports missing safe-host-tool configuration and separate restart/graph/
budget modes as remaining gate inputs; it returns 2 when coverage gates remain,
1 for observed failures, 0 only when the selected mode has no failure/gap. No
unexecuted case is recorded passed. Database engine/migration provisioning and
supervisor stop/join provenance belong to the enclosing operator gate. No
backend-matrix or process-attestation claim is made by this HTTP driver.

All new behavior remains unverified because the gate is CLOSED. No candidate,
provider, database, Docker, workflow, build, package or remote agent was started.

## Original OpenAI owned-source follow-up

Fresh ClientToolInput initializes graph_state=None and budget_snapshot=None.
Continue assigns the persisted claim.session_id rather than a new UUID. This
retains the runtime's graph/accounting continuation scope. No verification ran.
