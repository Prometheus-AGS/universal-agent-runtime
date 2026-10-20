# Function-First Local Qualification — Execution Report (Round 6 Redirect)

**Date**: 2026-10-18
**Contract**: `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/function-first-local-qualification-contract.md`
**Build identity** (frozen at start of execution):
- source_tree `sha256:f2e2025bda4c77059bf67720a6fc41a2d64949bc59e7651e0dca14f42668cc6c`
- fixture_tree `sha256:16694a84fc3f998194a5598e04e708240f8b832bf14d474f12a2f3855b8362a6`
- image `sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`
- HEAD `38f0b5d8` + uncommitted round edits.

## Native checks executed (existing fixture set, no new matrix)

| Source | Command | Result |
|---|---|---|
| `tests/bauar_skill_selection.rs` | `cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test bauar_skill_selection` | 1/1 PASS — `real_selection_preserves_stored_document_and_embedding` |
| `tests/agent_threads.rs` | same | 10/10 PASS (4.14s) |
| `tests/anthropic_driver_tests.rs` | same | 12/12 PASS (driver, not live loopback 200) |
| `tests/agui_golden.rs` | same | 1/1 PASS (schema validation) |
| `tests/a2a_thread_service.rs` | same | 1/4 PASS; 3 FAIL (model-dependent — see Observed failures) |
| `tests/context_history_integrity.rs` | same | 20 of 33 ok + 2 ignored (frozen-source diagnostics) before SIGABRT in `run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch` |
| `tests/dot_memory_owner_contract.rs` | same | 0/2 FAIL (owner-boundary) |
| `tests/builtin_db_registration.rs` | same | COMPILE ERROR (pre-existing — not in redirect scope to fix) |
| `tests/mcp_child_environment.rs` | same | NOT RUN (requires `test-probes` feature, out of scope) |
| `tests/bauar_resource_grants.rs` | same | 7/8 PASS; 1 FAIL (`grants_pin_owner_run_destination_revision_lease_and_action` — owner-binding revision lease issue) |
| `tests/bauar_secret_projection.rs` | same | 3/3 PASS (secret projection) |
| `tests/bauar_identity_boundary.rs` | same | 0/0 (no tests in target) |
| `tests/agent_policy_intersection.rs` | same | 4/4 PASS (agent policy) |
| Live AG-UI client capture (with user JWT) | `python3 /tmp/uar111-r6-r9-agui-live.py` (boots a node via the fixture harness, POSTs to `/api/uar/ag-ui/agents/default-agent` with `threadId`/`runId`/`protocolVersion` + a user JWT + a user message, parses the SSE stream with chunk-level deadline) | 200 OK; 2 events in order (`RUN_STARTED` with threadId + runId matching the request, `RUN_ERROR` at `credential_binding`). 200 + chunk identity + order is the **authenticated transport subpart only**; the contract R9 requires no-login. |
| Live AG-UI client capture (no-login) | `python3 /tmp/uar111-r9-noauth.py` (same path, no Authorization header) | **401** (the AG-UI route rejects no-login). The contract's "loopback, no user login" 200 + chunk identity + order is NOT met; the route requires authentication. |
| R6 + R7 live demo (with user JWT and no-login variants) | `python3 /tmp/uar111-r6-r7-live.py` (boots a node, POSTs to `/v1/chat/completions` and `/v1/messages`, each twice: with_user_jwt and no_auth_header) | R6_OpenAI/with_user_jwt: 422 `model_not_verified`; R6_OpenAI/no_auth_header: 401 (route requires authentication); R7_Anthropic/with_user_jwt: 403 `credential_source_denied`; R7_Anthropic/no_auth_header: 401 (route requires authentication). Contract bar (200 + agent-reflecting reply) not met; product/config gaps recorded. |

**Adjacent product issues surfaced** (NOT consumed from the cycle pool
yet — recorded for awareness):
- `tests/context_history_integrity.rs` `run_manager_persists_pending_tool_calls`
  STACK OVERFLOW (a real product issue in the run manager's tool-call resume
  path; fixing it would touch the R21 row's resume sub-criterion).
- `tests/dot_memory_owner_contract.rs` 0/2 FAIL (owner-boundary contract
  failure; not the R4 sub-criterion).
- `tests/bauar_resource_grants.rs` 1/8 FAIL
  (`grants_pin_owner_run_destination_revision_lease_and_action` — owner-binding
  revision lease issue; not the R11 sub-criterion).

**Evidence receipts**: per-row evidence paths are recorded under each KBD row
in `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/function-first-r6-kbd-rows.md`.

**Decisive rows (this continuation + prior)**:
- R6 (OpenAI) — **PARTIALLY GREEN**: native secret-projection 3/3 PASS;
  live R6 attempt with_user_jwt → 422 `model_not_verified`; no_auth_header
  → 401 (route requires authentication; the contract's "no login" path is
  blocked). Contract bar (200 + agent-reflecting reply) not met. Product
  gap: route requires auth + verified model provider.
- R7 (Anthropic) — **PARTIALLY GREEN**: native driver 12/12 PASS; live R7
  attempt with_user_jwt → 403 `credential_source_denied`; no_auth_header →
  401. Contract bar not met. Product gap: route requires auth + Anthropic
  source authority.
- R9 (AG-UI streaming) — **PARTIALLY GREEN**: with_user_jwt → 200 + chunk
  identity + order on the authenticated transport subpart only;
  no_login → 401. The contract R9 (loopback, no user login) bar is not
  met. Product gap: AG-UI route requires auth; no no-login mode.
- Plus earlier agent_threads 10/10 → R1/R2bcd/R10/R11/R20/R22 partial
  coverage, bauar_skill_selection 1/1 → R2ab, bauar_resource_grants 7/8
  → R5/R11 partial, bauar_secret_projection 3/3 → R6 partial,
  agent_policy_intersection 4/4 → R23 partial.

**KBD row table** (`.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/function-first-r6-kbd-rows.md`):
**0 contract-level PASS · 11 PARTIALLY GREEN · 11 UNVERIFIED · 9 observed failures / gaps preserved separately** (A2A 3/4, dot_memory_owner 2/2,
context_history_integrity SIGABRT, builtin_db compile, bauar_resource_grants
1/8, **R6 product gap — route requires auth; no-login blocked; model
unverified**, **R7 product gap — route requires auth; phase-user principal
lacks Anthropic source authority**, **R9 product gap — AG-UI route requires
auth; no-login 401; with-JWT 200 + RUN_ERROR at credential_binding is
only the authenticated transport subpart**, **R5/R11 product gap — 4 REST
endpoints return 401 without an Authorization header; no no-login REST
mode is configured**).

## Decision (this execution)

The function-first local qualification is the new immediate objective per the
operator redirect. The frozen contract's bar (loopback 200 + agent-reflecting
reply for R6/R7; live AG-UI client 200 stream with chunk identity + order
for R9; model-side prompt reflection for R1; memory write/recall for R4; KB
lifecycle for R8/R16-R20; tool call for R3; context strategy + trimming
for R21; budgets for R22; coordinator + 2 subagents for R23; multimodal
modalities for R13; storage modes for R14/R15; provider update live-routing
for R12; local REST create/configure for R5/R11) requires loopback
demonstrations that the residual budget must support. The supporting
native tests are PARTIAL evidence only — they cover the driver / harness
path, not the loopback 200 bar.

**Status totals (truthful — observed failures preserved separately, no
0-FAIL claim)**: **0 contract-level PASS · 11 PARTIALLY GREEN · 11
UNVERIFIED · 9 observed failures / gaps preserved separately** (A2A 3/4,
dot_memory_owner_contract 2/2, context_history_integrity SIGABRT,
builtin_db compile, bauar_resource_grants 1/8, **R6 product gap**,
**R7 product gap**, **R9 product gap**, **R5/R11 product gap — 4 REST
endpoints return 401 without an Authorization header; no no-login REST
mode is configured**).

R9 is **PARTIALLY GREEN** (not contract-level PASS): the 200 + chunk
identity + order was the authenticated transport subpart; the contract
requires no-login. The R9 no-login attempt returned 401 — the route
requires authentication. No admin authority was added; no silent
service-account injection.

R6 is **PARTIALLY GREEN**: the route requires authentication; the
harness's model "openai/gpt-4o-mini" is not verified. R7 is **PARTIALLY
GREEN**: the route requires authentication; the `phase-user` principal
lacks the Anthropic source authority. Both are honest product/config gaps;
no admin authority was added.

The two adjacent product issues (run_manager stack overflow;
owner-boundary contract failure) are recorded but not consumed from
the cycle pool — a repair cycle would only be used if the operator
authorizes a within-scope fix and a contract sub-criterion is gated on
that fix.

## Reconciliation (numerical — replaces the stale "comfortable" framing)

- **Before continuation** (round-6 close at goal `2f270092` `closedAt`
  1792357346 ≈ 2026-10-18 21:02:26 UTC): timeUsedSeconds **4,447**,
  tokensUsed **32,839,196**, repair cycles **2 of 2 consumed**, tool
  calls **≈ 28** (per the goal close fields and the prior report's
  narrative).
- **Operator-authorized additive** (2026-10-18): **+90 min,
  +60 tool calls, +2 repair cycles**.
- **New total ceiling**: **240 min / 130 tool calls / 4 repair
  cycles**.
- **This continuation** (operator "Continue the existing approved
  allowance"): the added tool calls are the 4 extra native test runs
  (`bauar_resource_grants`, `bauar_secret_projection`, `bauar_identity_boundary`,
  `agent_policy_intersection`), the 3 live demo runs (the with-JWT R9
  demo, the R6+R7 with/without-JWT demo, the no-login R9 demo), plus the
  KBD row table + execution report file edits (writes + edits). The
  best-estimate added tool calls for this continuation are
  **≈ 20–25** (the operator can verify the exact count from the
  transcript). The best-estimate added wall time for this continuation
  is **≈ 30–60 min** (the 4 native test runs together took ~98 s per
  the cargo test reports; the 3 live demo runs each took ~30–120 s
  plus the 2 s per file edit).
- **Estimated cumulative under the new ceiling** (rough, not a count
  the system exposes directly): **≈ 80–100 min** wall, **≈ 48–53
  tool calls**, **2 repair cycles consumed** (0 in this continuation).
- **Estimated residual under the new ceiling**: **≈ 140–160 min**,
  **≈ 77–82 tool calls**, **2 repair cycles remaining**.

The previous "comfortable" framing is replaced by these numerical
estimates. No silent time / call / cycle extension. No silent
service-account injection. No admin authority added. No commit, push,
tag, release, or deploy. CI/CD container image preserved (the build
that produced `sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`);
focused existing know-me.tools / The Boss native site integration smoke
deferred until a row reaches a state that needs it.

## Next incomplete frozen functional criterion to advance (no broad test suite)

The operator said "advance the next incomplete frozen functional criterion;
use native routes, not another broad test suite." R1 (Agent system
prompt honored) is the next decisive one — the native test
`child_history_fork_none_contains_only_the_delegated_prompt` (in
`tests/agent_threads.rs`) covers the routing/preservation aspect; the
full model-side prompt reflection requires a reachable model. The
remaining criteria all need a live model or admin credential or the
`test-probes` feature; the honest path forward is to attempt R1 with a
focused harness turn against the local Peer mock (which returns a
canned response) and assert the model response content reflects the
system prompt. If the Peer mock suffices, R1 can move from
PARTIALLY GREEN to a contract-level demonstration. If not, R1 stays
UNVERIFIED with the recorded reason.
