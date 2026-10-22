# Remaining local gates — source-only next plan

Phase Execute. Source inspection/planning only. No program, import, network call,
probe, test, check or build ran. The lead owns build005 and the scoped functional
test/fix/retest loop. This document changes no executable or production source.

The uncomfortable thing: the remaining CLI modes exist, but the current local
supervisor cannot execute them, and the graph/budget assumptions in the driver
are not yet bound to the real production paths. Naming these modes is not proof
that they are ready or that an arbitrary registered agent enables them.

## Exact existing mode surface

All modes retain required base/provider URLs, two distinct credential env names,
agent ID, provider-qualified --anthropic-model and report. Leave --openai-model
unset for graph/budget variants so Driver selects the scenario's actual agent.
All five interfaces are selectable together.

| Mode | Additional existing inputs | Current assertion |
| --- | --- | --- |
| restart-seed | --restart-state, --instance-identity | Save each protocol's pending results/native identity without submitting results. |
| restart-resume | Same state file, distinct instance identity | Submit after restart; complete and replay without another provider contact. |
| budget | --budget-agent-id | Initial caller handoff succeeds; resumed turn fails with a budget-specific error and at most two provider contacts. |
| budget-restart-seed | Budget agent, state file, identity | Save pending state under budget-bound agent. |
| budget-restart-resume | Same state, budget agent, new identity | Cold result fails at retained budget; identical failed claim makes no new provider contact. |
| graph | --graph-agent-id, --graph-before-marker, --graph-after-marker | Currently expects provider sequence before/before/after. This assertion needs amendment below. |
| graph-restart-seed | Graph inputs, state, identity | Save graph caller handoff. |
| graph-restart-resume | Same graph inputs/state, new identity | Cold completion/replay plus the same currently incorrect graph sequence assertion. |

Current supervisor always creates fresh configuration, JWTs, store and peer; runs
only matrix; stops every child in finally. It has no attach/restart mode. Running
remaining driver commands after that supervisor returns would find no candidate
or peer. Reusing a fresh supervisor would change owner/config/store and prove
nothing about cold continuation.

## First bounded extension: genuine cold ordinary continuation

Amend the existing supervisor rather than adding another framework:

1. Keep the already provisioned provider/MCP peer, same HTTP port, test JWTs,
   config bytes, private HOME/CWD, actual MCP names and SurrealKV path alive.
2. Run restart-seed for all five interfaces with a private state file and a
   supervisor-created initial identity containing candidate PID/generation.
   Retain the driver report and provider/MCP counts.
3. Stop **only** the candidate via SIGTERM and join it. Require confirmed join
   and expected clean exit before another process may acquire that store. No
   forced-kill result earns cold-restart acceptance. Retain exact lifecycle receipt.
4. Start the same prebuilt binary with identical config/environment/ports/CWD.
   Keep the peer alive and do not reset its trace/counter. Run readiness inside
   the same proposed gate. Persisted provider verification should remain current;
   do not mutate provider config or reissue verification merely to fabricate it.
5. Run restart-resume using the same state and credentials with a distinct
   supervisor-recorded process identity. Require all five native completions and
   exact retries without provider reexecution. Finally stop/join candidate then peer.

The driver identity string alone is not process evidence. The supervisor's actual
stop/join/PID/generation receipts bind it. This first extension needs no graph,
budget or product configuration surface beyond the existing confirmed setup.

## Real graph surface and required driver amendment

Source bindings:

- server.rs installs defaults::orchestrator_graph() with with_agent_graph.
- RunManager::uses_agent_graph is true only for artifact ID **orchestrator-agent**
  while that host-owned graph is installed. POSTing another AgentArtifact, changing
  runtime.entry or adding an invented graph extension does not select this path.
- defaults::orchestrator_graph is RouterNode("router") followed by one specialist
  AgentNode: general-purpose or rust-reviewer. Its router prompt starts “Route Rust
  implementation, correctness, or safety questions to rust-reviewer.” RouterNode
  sends a real tool-free model request. There is no configurable two-model-node
  graph API/file setting in these inspected startup sources.
- Default/orchestrator definitions are refreshed on startup. Specialist definitions
  are inserted only when absent, so an explicit persisted update to general-purpose
  can survive restart. Existing GET/PUT /api/agents/{id} supports whole artifacts.

Amend graph acceptance to this **existing** graph, without adding product graph
configuration solely for a test:

- Select --graph-agent-id orchestrator-agent (and use its actual route on all five
  transports). Update the existing general-purpose artifact through its real API
  to the verified loopback provider/model and a unique worker prompt marker;
  preserve unrelated fields. Save the before/after artifact receipt.
- Teach the real controlled peer's graph scenario to recognize the source-defined
  tool-free router request and return exactly `general-purpose`. The current peer
  returns generic no-call text, which only exercises router fallback.
- At the selected worker, return the caller function, then final text when results
  arrive. Expected actual provider sequence is **router / worker / worker**, with
  one router request total, one initial worker request and one resumed same-worker
  request. There is no “after” model node to assert. Use observed system/active-node
  prompt identity, not user content or a guessed graph cursor field.
- Require no repeated router or skipped worker across both warm and genuine cold
  continuation; preserve provider trace and graph receipts through candidate restart.
  Do not credit the existing before/before/after assertion as this proof.

This requires bounded fixture/driver amendments plus specialist provisioning in
the supervisor. The runtime graph path itself already exists; no new permissive
policy, private graph executor or fake unit callback is part of this plan.

## Real budget surface and required numeric/proof amendment

ThreadBudgets::from_artifact strictly decodes **extensions.budgets** on a persisted
AgentArtifact. Supported keys include max_tokens_per_turn,
max_tokens_per_session, max_tool_calls_per_turn, max_cost_per_session_usd,
timeout_seconds and rate_limit (requests_per_minute/tokens_per_minute). Do not put
budgets on AgentPolicy or invent max_model_requests. Provision a dedicated ordinary
agent through POST /api/agents with a real max_tokens_per_session ceiling and keep
it unchanged across warm/cold runs.

The existing driver says “below 10 tokens” because the peer currently reports
2 input + 3 output per turn. That ceiling need not admit even the first real
prompt. ModelCallBudget::begin_call reserves the actual final-wire input/output
bound when available; otherwise it uses source-catalog context/output bounds.
It does not reserve only the peer's eventual 5-token usage. The peer's OpenAI and
Anthropic usage objects are currently hardcoded; `_control` has no usage override.

Required bounded amendments before claiming a runnable token-budget gate:

- Add explicit synthetic **provider-wire usage control** to the real peer for
  the budget scenario. Emit matching OpenAI/Anthropic usage values on the actual
  HTTP response. This changes no runtime ledger, private counter or persisted row.
- Use a source-catalog-backed provider/model identity with its endpoint explicitly
  redirected to the same loopback peer; e.g. source-supported openai/gpt-4o-mini
  with the isolated test key. Keep UAR_LOCAL_ONLY enforced. Verify this actual
  configuration through the protected provider test endpoint. A custom `fixture`
  identity must not be assumed to have catalog reservation/pricing metadata.
- Bind first-call reservation R1 and resumed-call reservation R2 from the real
  supported model/final-wire accounting contract; do not guess numeric limits.
  Select a ceiling L and controlled first settled usage S satisfying
  **R1 <= L, R2 <= L, S + R2 > L**. The second inequality matters: otherwise the
  second request would fail even if cumulative usage had incorrectly reset.
  Choose provider usage within the admitted response contract and retain those
  exact wire receipts. If the existing metadata cannot establish R1/R2, author
  the smallest supported diagnostic observation in the gate before choosing L;
  do not bypass reservation or use an arbitrarily tiny token ceiling.
- Warm mode must prove the first handoff succeeds and resumed admission fails
  specifically because the settled prior usage remains charged. Cold mode uses
  the same exact pending batch/config/store after an actual candidate join/restart,
  then requires the same budget-specific failure and no provider contact on the
  repeated claimed batch. Retain real session identity and provider usage counts.

ClientToolBudgetSnapshot already retains limits, scoped token/cost usage,
cumulative activity and windows. The plan tests that production snapshot/ledger
path; it does not directly author or mutate a snapshot. The short request-rate
window can separately test retained activity, but it is not a substitute for the
requested cumulative token-budget evidence.

## Proposed order and authorization boundary

1. Resolve current five-interface matrix findings under the lead's scoped loop.
2. Author the bounded ordinary cold-restart supervisor extension and exact gate.
3. Author real-router graph fixture/assertion/provisioning amendments; warm graph
   then cold graph under separately inventoried concrete gate commands.
4. Author controlled wire-usage and reservation-bound budget setup; warm then cold
   budget after its numeric admission conditions are source-bound.

Each new gate must include provisioning, readiness, lifecycle, observations,
reports and final child joins inside its exact proposed command. No standalone
implicit preflight, global /data write, auth/governance disable, unverified model
flag or fake durability acknowledgment is authorized by this plan. No command in
this document was executed. Remaining fixture/assertion/provisioning work must be
implemented before presenting those gates as ready to run.
