# Function-First Local Qualification Contract — Round 6 Redirect

**Date**: 2026-10-18
**Context**: Round 6 (150 min / 70 calls / 2 repair cycles) redirected from
deployment-wide 1.1.1 certification to a function-first local qualification
contract (per the operator redirect, 2026-10-18). Round 1–5 reports and
receipts preserved byte-identical and untouched. The in-flight Docker build
result is preserved: image
`sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`
(the corrected source compiles cleanly). No push, tag, release, or deploy is
authorized.

## KBD transition record (redirect, 2026-10-18)

- Round-6 phase: certification/credential-migration acceptance (NOT
  READY in round 5, deferred per redirect).
- This contract: function-first local qualification (replacement scope
  under the same allowance; preserves cumulative accounting).
- Accounting preserved (not silently reset): round-6 ceilings 150 min /
  70 tool calls / 2 repair cycles (both consumed in round 6); goal clock
  was ~2840 s at the redirect; remaining budget ≈ round-6 residual. The
  remaining work for this contract must fit that residual — the contract
  table is published first, then native checks are executed incrementally
  until the budget is consumed; any remaining rows stay UNVERIFIED and go
  to a backlog. No self-renewal of the budget, no redefinition of done
  around a blocker.

## Source / build identity (frozen for this contract)

- source_tree / fixture_tree: frozen at the start of contract execution by
  the existing `source_identity` procedure (`git ls-files --cached
  --others --exclude-standard` over SOURCE_PATHS / FIXTURE_PATHS).
- Image: `sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`
  (rebuilt from the corrected source after the type-path correction in
  round 6; embedded build identity = this digest).
- Persistence: representative flows exercised on ONE Surreal mode; the
  storage-dependent rows (R14, R15) are repeated on the other mode.
  PostgreSQL is deferred (per the redirect).
- Auth: local loopback, no user login. Flint is the intended cloud-user
  gate (deferred). Routes that currently require a verified owner or
  admin key are NOT silently bypassed — failures are reported as
  product/contract gaps and fix only within this scope.

## Common fields for every row

- **Criterion ID**: R# (1–23).
- **Setup / input / config**: explicit local-loopback inputs; distinguish
  "no user auth" from "database / provider credentials" (no third-party
  cloud auth in this contract).
- **Observable expected output**: a measurable output (HTTP status +
  body / state file / log trace / assembled context) and the persisted
  state after the operation.
- **Existing executable command or demo**: the specific entry point used
  (existing fixture harness, existing test target, existing demo) — no
  new test matrix; reuse one fixture set.
- **Evidence file + source/build identity**: where the output is recorded
  (log path under the phase dir) and the build identity it refers to.
- **Timeout**: per-row budget (each row independent).
- **Repair allowance (per-row narrative cap)**: the per-row `Repair
  allowance 1` line is the narrative cap on attempts for that row ONLY;
  all per-row attempts **share the global cycle pool** (2 base + 2
  operator-authorized additive = **4 total repair cycles** for this
  UAR session). Per-row repair attempts beyond that global pool are not
  authorized; failures become UNVERIFIED backlog. This is not a
  redefinition of the budget — the per-row "1" was always intended as
  a row-level cap, not an additional global allocation.
- **Initial status**: UNVERIFIED until executed (a written criterion is
  not an executed PASS).

## Cumulative accounting (KBD-tracked; preserved across the redirect)

- Round-6 (cancelled goal 2f270092) closed with: timeUsed 4447 s (~74
  min), tokensUsed 32 839 196, repair cycles 2/2, call count ~28 (per
  the round-6 report's narrative estimate).
- Operator-authorized additive continuation (2026-10-18): +90 min,
  +60 tool calls, +2 repair cycles. No reset.
- New total ceiling: 150+90=**240 min**, 70+60=**130 tool calls**,
  2+2=**4 repair cycles**.
- Residual under the new ceiling: ~166 min, ~102 tool calls, 2 repair
  cycles. The contract and report work will be tracked against this
  residual; any per-row fix consumes from the 2 remaining cycles (one
  per row is a narrative cap, NOT a global allocation).

## Fixed 23-row exit-criterion table

R1 — Agent system prompt honored
- Setup: configure a local agent with a distinctive system prompt;
  start a single turn; no login.
- Expected: the assembled request to the underlying model contains the
  prompt verbatim; the model's reply reflects it.
- Command: fixture harness (`scripts/release/uar_1_1_1_support.py`
  `Resources`/`RuntimeLease`) + a captured model request trace.
- Timeout 60s. Repair allowance 1 (prompt templating only).

R2 — Skill: assignment, activation strategy, agent-specific
preservation, AG-UI stream identity (refines the skills row;
subcriteria a/b/c/d from the operator clarification)
- (a) Assign a skill to an agent and read the assignment back via the
  registry / config API.
- (b) Enumerate and FREEZE the actually implemented per-agent skill
  activation strategies (this list becomes the fixed input). Then
  prove: a relevant skill activates and is used; an unrelated or disabled
  skill does NOT activate (verified in the run trace / AG-UI chunks).
- (c) Capture activation in the assembled model input / run trace and
  show the agent-specific activation-strategy settings are preserved
  for the intended child/team agents WITHOUT cross-agent leakage
  (team-A's settings must not appear in team-B's input).
- (d) On a real AG-UI client stream, observe activation chunks with
  correct run / agent / skill identity and ordering DURING the run (a
  server log alone is insufficient; explicit skill loading cannot stand
  in for automatic strategy activation).
- Command: `tests/agent_threads.rs` + `tests/bauar_skill_selection.rs` +
  `tests/agui_golden.rs` extended by the existing fixture harness for
  subcriteria (b)/(c); AG-UI stream capture from the existing AG-UI
  client.
- Timeout 90s. Repair allowance 1 (activation strategy wiring only).

R3 — Tool: agent calls a configured tool
- Setup: configure a tool on the agent; run a turn that requires it.
- Expected: tool invocation in the run trace; final reply reflects
  the tool result.
- Command: `tests/builtin_db_registration.rs` + harness.
- Timeout 60s. Repair allowance 1.

R4 — Memory: write/recall across turns
- Setup: turn 1 writes a memory entry; turn 2 references the same
  scope; same thread / context.
- Expected: the assembled input for turn 2 includes the entry written
  in turn 1; the reply reflects recall.
- Command: existing harness loopback; capture assembled input.
- Timeout 60s. Repair allowance 1.

R5 — Knowledge base attachment (linkage visible to agent)
- Setup: create a KB, attach to the agent config; KB itself empty.
- Expected: the agent config shows the KB link; the empty KB returns
  no results in a probe (lifecycle itself is exercised under R16–R20).
- Command: harness POST to the agent-config endpoint; GET to read back.
- Timeout 30s. Repair allowance 0 (no fix; this row only asserts
  linkage).

R6 — OpenAI interface turn
- Setup: request via the OpenAI chat-completions-compatible endpoint;
  local loopback; no login.
- Expected: 200 response; reply content reflects the agent + tools.
- Command: existing harness curl/Http against the OpenAI route.
- Timeout 60s. Repair allowance 1.

R7 — Anthropic interface turn
- Setup: request via the Anthropic messages-compatible endpoint; local
  loopback; no login.
- Expected: 200 response; reply reflects the agent.
- Command: `tests/anthropic_driver_tests.rs` + harness.
- Timeout 60s. Repair allowance 1.

R8 — A2A interface turn
- Setup: A2A message endpoint; local loopback; no login.
- Expected: 200 response; reply reflects the agent.
- Command: `tests/a2a_thread_service.rs` + harness.
- Timeout 60s. Repair allowance 1.

R9 — AG-UI interface turn (streaming)
- Setup: AG-UI client stream; local loopback; no login.
- Expected: 200 + streaming chunks; identity and ordering correct (see
  R2(d)).
- Command: `tests/agui_golden.rs` + AG-UI client.
- Timeout 90s. Repair allowance 1.

R10 — Registry-backed A2UI rendering and action handling (refines
A2UI; subcriteria (d) + action)
- Setup: a registry-backed A2UI surface is rendered (component tree
  from the registry) and an action callback mutates state.
- Expected: rendered chunks with correct run/agent identity; the action
  fires and state changes per the registry semantics.
- Command: `tests/agui_golden.rs` + AG-UI client; observe chunks AND
  state after the action.
- Timeout 90s. Repair allowance 1.

R11 — Local REST create/configure of agents, tools, and MCP servers
without user login
- Setup: POST/GET to the configuration endpoints; no login header.
- Expected: 200 on create; 200 on read-back; the new objects are
  queryable; MCP server is reachable on the configured endpoint.
- Command: harness POST/GET against the local REST surface.
- Timeout 60s. Repair allowance 1.

R12 — Liter-LLM provider/model configuration with immediate real-time
use (edge: next-request routing)
- Setup: configure a provider + model via REST; immediately issue a
  request without restarting anything.
- Expected: the next request routes through the newly configured
  provider/model; provider response observed in the trace.
- Command: harness; two consecutive POSTs to the config + agent routes.
- Timeout 60s. Repair allowance 1.

R13 — Multimodal turns (modalities stated precisely)
- Setup: model advertised as multimodal; send a request with the
  supported modalities (text + at least one of: image, audio). The
  exact set of supported / exercised modalities is recorded in the
  evidence (not a generic "multimodal works" claim).
- Expected: assembled request contains the listed modalities; the model
  returns content that references them; the capability boundary is
  explicit in the row evidence.
- Command: harness with the chosen modality inputs; assert the model's
  capability table for that model.
- Timeout 60s. Repair allowance 1.

R14 — Embedded SurrealKV representative flow
- Setup: backend = embedded SurrealKV; configure KB + agent; run a
  retrieval + a turn.
- Expected: reads/writes succeed against the embedded store; the
  representative flow passes.
- Command: harness with `backend="local"` (embedded SurrealKV via the
  R4 unit fix + content-versioned config + credential fix).
- Timeout 90s. Repair allowance 1.

R15 — Remote SurrealDB representative flow
- Setup: backend = remote SurrealDB; same flow as R14.
- Expected: same observable outcomes; storage-dependent checks
  repeated (per the redirect).
- Command: harness with `backend="surreal"`; configured against the
  pinned SurrealDB image.
- Timeout 90s. Repair allowance 1.

R16 — KB create/list + upload/index
- Setup: create a KB; list it; upload a document; index.
- Expected: KB list returns the new KB; document appears indexed (search
  returns it after upload/index).
- Command: harness against KB endpoints.
- Timeout 60s. Repair allowance 1.

R17 — KB embeddings + vector search
- Setup: after R16; query via the embedding/vector search route.
- Expected: top-k includes the indexed document with similarity /
  score recorded.
- Command: harness vector-search call.
- Timeout 60s. Repair allowance 1.

R18 — Agent use of KB with source evidence
- Setup: a turn that requires KB content; KB populated.
- Expected: the reply cites / reflects the source; the assembled input
  includes the retrieved chunk; the model uses it.
- Command: harness + model-request capture.
- Timeout 60s. Repair allowance 1.

R19 — Document deletion: stale vectors absent (direct search + fresh
agent retrieval) + KB deletion
- Setup: delete a document; then (i) direct vector search returns no
  hits for that doc; (ii) a fresh-agent retrieval that should have seen
  the doc does NOT; then delete the KB.
- Expected: after each deletion the doc is unreachable; after KB
  deletion the KB is gone.
- Command: harness delete + search + retrieval + KB delete.
- Timeout 90s. Repair allowance 1.

R20 — Persistence / reconnect without resurrection
- Setup: populate state (KB, memory, threads); simulate a restart (or
  open a fresh store against the same volume); verify state is present
  AND that no "phantom" / "ghost" state reappears from prior sessions.
- Expected: persisted objects present; no resurrection of deleted
  objects across reconnect.
- Command: harness + persistent volume + reconnect.
- Timeout 60s. Repair allowance 1.

R21 — Context strategy selection + trimming that preserves protected
content under actual model limits
- Setup: an agent with system prompt + history + skills + memory + RAG
  input exceeding the model context window; the strategy selects +
  trims.
- Expected: the assembled input fits the real model window + reserved
  output; protected items (system, history, skills, memory, RAG) are
  preserved according to the strategy; trim order is recorded.
- Command: harness with a deliberately oversized input; capture
  assembled input.
- Timeout 60s. Repair allowance 1.

R22 — Budgets (input + reserved-output, tool, child, root) with
truthful unknown-liability refusal (edge: refuse BEFORE the action +
aggregate child usage)
- Setup: drive a run that exhausts each budget class in turn; capture
  the refusal.
- Expected: when a budget would be exceeded, the system REFUSES BEFORE
  performing the action and reports the budget class; child usage is
  aggregated into the parent.
- Command: harness driving the budget hooks.
- Timeout 60s. Repair allowance 1.

R23 — Coordinator + 2 subagents + isolation / bounded concurrency
(edges: joined results + cancellation no-orphan + no cross-talk /
deadlock / lost results)
- Setup: a coordinator spawns two subagents; one succeeds, one is
  cancelled; meanwhile a second concurrent thread on the same agent
  type runs without cross-talk.
- Expected: joined results returned to the caller; the cancelled child
  leaves NO new side effects; the parallel thread does not deadlock or
  lose its result; per-thread / per-conversation isolation holds.
- Command: harness spawning the team + a parallel thread; assert
  cancellation effects and isolation.
- Timeout 90s. Repair allowance 1.

## Edge checks (mapped to rows, per the operator specification)

- Provider update routes the very NEXT request without restart → R12.
- Deleted document disappears from direct vector search AND
  fresh-agent retrieval → R19.
- Assembled context including required instructions fits the real model
  window plus reserved output → R21.
- Budget exhaustion refuses BEFORE the action AND aggregates child usage
  → R22.
- Team children return joined results AND cancellation terminates with no
  new effects → R23.
- A2UI actually renders AND handles an action → R10 (and R2(d)).

## Reporting format (KBD rows per row, PASS / FAIL / UNVERIFIED)

Each row's evidence block in the consolidated report will record:
PASS | FAIL | UNVERIFIED; exact source/image identity (`38f0b5d8` +
`source_tree` / `fixture_tree` + `image
sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`);
command / route; observed result; receipt path under
`.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/`. Blocker =
shortest next action.

## Frozen identity for this contract (recorded at start of execution)

- source_tree: `sha256:f2e2025bda4c77059bf67720a6fc41a2d64949bc59e7651e0dca14f42668cc6c`
- fixture_tree: `sha256:16694a84fc3f998194a5598e04e708240f8b832bf14d474f12a2f3855b8362a6`
- Image: `sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`
  (rebuilt from the corrected source after the type-path fix in the redirect
  build; embedded build identity = this digest)
- HEAD: `38f0b5d8` (uncommitted round edits on top)
