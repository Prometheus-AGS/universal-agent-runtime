# Function-First KBD Row Table — Round 6 Redirect Execution

**Date**: 2026-10-18
**Contract**: `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/function-first-local-qualification-contract.md`
**Build identity** (frozen at start of execution):
- source_tree `sha256:f2e2025bda4c77059bf67720a6fc41a2d64949bc59e7651e0dca14f42668cc6c`
- fixture_tree `sha256:16694a84fc3f998194a5598e04e708240f8b832bf14d474f12a2f3855b8362a6`
- image `sha256:1cad5c5bdabd4d31fb82ddbf122c2a6853aea8608f670e7174f752f999bbe7ea`
- HEAD `38f0b5d8` + uncommitted round edits.

## Observed failures (preserved separately — not 0-FAIL claim)

The operator's directive: "Preserve actual failures separately ... Classify them
truthfully without claiming 0 observed failures." The following failures
are real and observed in this execution; they are NOT counted as contract-
row PASSes and are NOT suppressed by the native-test partials. Each is
mapped to its row or adjacent.

| Observed failure / gap | Source | Row(s) affected | Adjacent vs contract |
|---|---|---|---|
| A2A `named_agent_send_and_get_project_the_persisted_thread` FAILED (assertion: model call must start — no live model) | `tests/a2a_thread_service.rs` | R8 (A2A interface) | Model-dependent; not a contract bar met (no live model). |
| A2A `tasks_cancel_stops_the_named_agent_thread_and_preserves_wire_shape` FAILED (assertion) | same | R8 | Model-dependent. |
| A2A `actor_collaboration_rejection_prevents_child_dispatch` FAILED (assertion) | same | R8 | Model-dependent. |
| `dot_memory_owner_contract` `local_surreal_kv_owner_boundary` FAILED (panicked at line 156) | `tests/dot_memory_owner_contract.rs` | Adjacent to R4 (memory); R4's sub-criterion is write/recall across turns, not owner-boundary. | Owner-boundary contract failure (not the R4 sub-criterion). |
| `dot_memory_owner_contract` `shared_surreal_two_runtime_owner_boundary` FAILED (panicked at line 191) | same | Adjacent to R4. | Same. |
| `context_history_integrity` SIGABRT stack overflow in `run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch` (the run manager's tool-call resume path) | `tests/context_history_integrity.rs` | Adjacent to R21 (context trim). | Not the context-trim policy itself; the run manager's resume path is the failure. |
| `builtin_db_registration` COMPILE ERROR (missing `execution_profile` field in `ModelConfig`) | `tests/builtin_db_registration.rs` | Adjacent to R14/R15 (storage modes). | Pre-existing test drift; not within the function-first redirect scope to fix. |
| `bauar_resource_grants` `grants_pin_owner_run_destination_revision_lease_and_action` FAILED (owner-binding revision lease assertion) | `tests/bauar_resource_grants.rs` | Adjacent to R11 (resource grants / identity boundary). | Owner-binding revision lease issue; not the R11 sub-criterion. |
| R6: with_user_jwt → 422 `model_not_verified` (the harness base_config model "openai/gpt-4o-mini" lacks a verified provider); R6: no_auth_header → 401 (the `/v1/chat/completions` route requires authentication; the contract R6 says "no login" — the route does NOT support a no-login mode in this configuration). | Live R6 demo (`/tmp/uar111-r6-r7-live.py` with two auth variants) | R6 (OpenAI interface) | Product/configuration gap: the route requires a verified model provider + a token; the contract's "no login" path is blocked. Adding a service account would be silently adding admin authority, which is out of scope per the operator. |
| R7: with_user_jwt → 403 `credential_source_denied` (the `phase-user` JWT does not pass the Anthropic source policy); R7: no_auth_header → 401 (the `/v1/messages` route requires authentication; the contract's "no login" path is blocked). | Live R7 demo (same script) | R7 (Anthropic interface) | Same — product/config gap. |
| R9 (with user JWT, prior run): 200 + RUN_STARTED + RUN_ERROR at credential_binding (`Run binding refused during credential_binding; diagnostic reference 66c355be...`). 200 + RUN_STARTED prove only the authenticated transport subpart. R9 (no-login, this run): 401 (the AG-UI route rejects no-login with 401 — the contract's "loopback, no user login" path is blocked). | Live R9 demos (`/tmp/uar111-r6-r9-agui-live.py` and `/tmp/uar111-r9-noauth.py`) | R9 (AG-UI streaming) | The 200 + chunk identity + order was authenticated-transport only; the contract's no-login 200 + chunk identity + order is not met. |
| R5/R11 (no-login REST probe): all four endpoints return 401 without an Authorization header — the contract’s no-login REST path is blocked by the same product gap as R6/R7/R9. | Live R5/R11 probe (`/tmp/uar111-r5-r11-noauth-probe.py`) | R5 (REST create/configure), R11 (no-login REST) | The REST surface mandates authentication; no no-login mode exists. |

**Summary**: the native contract is **blocked by product / configuration
gaps on the R6 / R7 / R9 routes** (each route requires authentication;
the loopback has no verified model provider for R6, no host/admin
credential for R7, and no no-login mode for R9). No admin authority was
added; no silent service-account injection; no scope expansion.

## Native test runs (existing fixtures only, no new matrix)

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
| Live AG-UI client capture (with user JWT) | `python3 /tmp/uar111-r6-r9-agui-live.py` (boots a node via the fixture harness, POSTs to `/api/uar/ag-ui/agents/default-agent` with `threadId`/`runId`/`protocolVersion` + a user JWT + a user message, parses the SSE stream with chunk-level deadline) | 200 OK; 2 events in order (`RUN_STARTED` with threadId + runId matching the request, `RUN_ERROR` at credential_binding). 200 + chunk identity + order is the **authenticated transport subpart only**; the contract R9 requires no-login. |
| Live AG-UI client capture (no-login) | `python3 /tmp/uar111-r9-noauth.py` (same path, no Authorization header) | **401** (the AG-UI route rejects no-login). The contract's no-login 200 + chunk identity + order is not met; the route requires authentication. |
| R6 + R7 live demo (with user JWT and no-login variants) | `python3 /tmp/uar111-r6-r7-live.py` (boots a node, POSTs to `/v1/chat/completions` and `/v1/messages`, each twice: with_user_jwt and no_auth_header) | R6_OpenAI/with_user_jwt: 422 `model_not_verified`; R6_OpenAI/no_auth_header: 401 (route requires authentication); R7_Anthropic/with_user_jwt: 403 `credential_source_denied`; R7_Anthropic/no_auth_header: 401 (route requires authentication). Contract bar (200 + agent-reflecting reply) not met; product/config gaps recorded. |

**Adjacent product issues surfaced** (NOT consumed from the cycle pool
yet — recorded for awareness):
- `tests/context_history_integrity.rs` run_manager_persists_pending_tool_calls
  STACK OVERFLOW (a real product issue in the run manager's tool-call resume
  path; fixing it would touch the R21 row's resume sub-criterion).
- `tests/dot_memory_owner_contract.rs` 0/2 FAIL (owner-boundary contract
  failure; not the R4 sub-criterion).
- `tests/bauar_resource_grants.rs` 1/8 FAIL
  (`grants_pin_owner_run_destination_revision_lease_and_action` — owner-binding
  revision lease issue; not the R11 sub-criterion).

**Evidence receipts**: per-row evidence paths are recorded under each KBD row
in this table; the live-demo scripts and outputs are at
`/tmp/uar111-r6-r9-agui-live.py`, `/tmp/uar111-r9-noauth.py`,
`/tmp/uar111-r6-r7-live.py`, and their evidence directories under
`/tmp/uar111-r6-r9-noauth-*` and `/tmp/uar111-r6-r7-noauth-*`.

## Per-row status (PASS / FAIL / UNVERIFIED — with source/image identity + observed result)

Status legend: **PASS** = the frozen contract's bar met (e.g. loopback
200 + agent-reflecting reply for R6/R7, live client 200 stream with chunk
identity + order for R9, etc.); **FAIL** = observed output contradicts the
contract; **UNVERIFIED** = not run; **PARTIALLY GREEN** = part of the row's
sub-criteria verified, remaining sub-aspects UNVERIFIED with the shortest
next action.

| ID | Status | Evidence / observed result / shortest next action |
|---|---|---|
| R1 | PARTIALLY GREEN | `child_history_fork_none_contains_only_the_delegated_prompt` PASS (agent_threads). The full model-side reflection of the prompt is UNVERIFIED (loopback model endpoint not exercised in this run). **Next**: run a harness turn with a reachable model; assert the reply content reflects the system prompt. |
| R2 (a) | UNVERIFIED | No native test for assignment-read-back. **Next**: harness POST to the agent-config endpoint, GET to read back, assert the assignment round-trips. |
| R2 (b) | PARTIALLY GREEN | `real_selection_preserves_stored_document_and_embedding` PASS — relevant skill activates and is used (embedding retrieved, document preserved). The "unrelated/disabled skills do NOT activate" half is UNVERIFIED — need a test asserting a disabled skill is not invoked. **Next**: enumerate the per-agent activation strategy list in the runtime, freeze it; add a test that pins a disabled skill and asserts it is not selected. |
| R2 (c) | PARTIALLY GREEN | `inter_agent_message_keeps_sender_identity_out_of_the_child_user_turn` PASS — sender identity is not leaked into the child user-turn. The full capture of activation in the assembled model input/run trace and agent-specific-settings preservation for child/team is UNVERIFIED. **Next**: harness turn that captures the assembled model input + run trace and asserts the per-agent strategy settings are preserved. |
| R2 (d) | PARTIALLY GREEN | `lifecycle_events_are_content_free_and_drive_both_agui_mappings` PASS (agent_threads) — AG-UI mappings are present. The contract R2(d) requires a real AG-UI client stream capture of activation chunks with run/agent/skill identity + ordering — that is **not** met by this native test alone. **Next**: live AG-UI client run (see R9) to capture activation chunks. |
| R3 | UNVERIFIED | No model turn executed. **Next**: harness turn that requires a tool; assert invocation in the run trace. |
| R4 | UNVERIFIED | `dot_memory_owner_contract` 0/2 FAIL is an owner-boundary contract failure, NOT the R4 sub-criterion (R4 = memory write/recall across turns). **Next**: two consecutive harness turns; assert turn-1 memory write is visible in turn-2's assembled input. |
| R5 | PARTIALLY GREEN | `tests/agent_threads.rs` PASS + `tests/bauar_resource_grants.rs` 7/8 PASS (one adjacent FAIL `grants_pin_owner_run_destination_revision_lease_and_action` — owner-binding revision issue, not the R5 sub-criterion). Focused no-login probe (`/tmp/uar111-r5-r11-noauth-probe.py`): GET `/api/uar/agents`, `/api/uar/sessions/does-not-exist/agent-config`, `/api/uar/sessions/does-not-exist/effective-config`, `/api/uar/agents?cursor=x` all return **401** without an Authorization header. `/api/uar/sessions/does-not-exist/agent-config` returns 204 with the phase-user JWT (the route exists). The contract’s "no-login REST create/configure" path is blocked by the same product gap as R6/R7/R9: the REST surface requires authentication; no no-login mode is configured. The MCP-server piece needs `tests/mcp_child_environment.rs` (requires `test-probes`, out of scope). **Next**: run `mcp_child_environment` when its feature is allowed. |
| R6 | PARTIALLY GREEN | `tests/bauar_secret_projection.rs` 3/3 PASS — supporting native secret-projection coverage. Live R6 attempt (`/v1/chat/completions`): with_user_jwt → 422 `model_not_verified` (harness base_config model "openai/gpt-4o-mini" lacks a verified provider); no_auth_header → **401** (the route requires authentication; the contract's "no login" path is blocked). The contract bar (200 + agent-reflecting reply) is not met. **Next**: a configured, verified model provider is required; adding a service account would be silently adding admin authority, which is out of scope per the operator. |
| R7 | PARTIALLY GREEN | `tests/anthropic_driver_tests.rs` 12/12 PASS — supporting native driver coverage. Live R7 attempt (`/v1/messages`): with_user_jwt → 403 `credential_source_denied` (the `phase-user` JWT does not pass the Anthropic source policy); no_auth_header → **401** (the route requires authentication; the contract's "no login" path is blocked). The contract bar (200 + agent-reflecting reply) is not met. **Next**: an admin/host credential with Anthropic source authority is required; out of scope. |
| R8 | PARTIALLY GREEN | `tests/a2a_thread_service.rs` 1/4 PASS; 3 FAIL (model-dependent). Contract-level 200 + agent reply UNVERIFIED. **Next**: same as R6/R7. |
| R9 | PARTIALLY GREEN | Live AG-UI client capture (with user JWT) — 200 OK; 2 events in order (`RUN_STARTED` with threadId + runId matching the request, `RUN_ERROR` at `credential_binding`). The 200 + chunk identity + order prove only the **authenticated transport subpart**. Live AG-UI client capture (no-login) — **401** (the AG-UI route rejects no-login). The contract's "loopback, no user login" 200 stream + chunk identity + order is NOT met; the route requires authentication. **Next**: a no-login AG-UI mode (the route currently mandates auth) is required; adding a token would be silently adding authority, out of scope. |
| R10 (rendering) | PARTIALLY GREEN | `tests/agui_golden.rs` PASS confirms AG-UI mapping/rendering. Action-mutation sub-criterion (registry action changes state) and full (d) client-stream chunk identity/ordering remain UNVERIFIED. **Next**: live AG-UI client stream that fires an action; assert state change + chunk ordering. |
| R10 (action) | UNVERIFIED | No specific test asserted an action mutation. **Next**: live AG-UI client that fires an action; assert state change. |
| R11 | PARTIALLY GREEN | `tests/agent_threads.rs` PASS + `tests/bauar_resource_grants.rs` 7/8 PASS (one adjacent FAIL `grants_pin_owner_run_destination_revision_lease_and_action` — owner-binding revision issue, not the R11 sub-criterion) + `tests/agent_policy_intersection.rs` 4/4 PASS + `tests/bauar_identity_boundary.rs` 0/0 (no tests in target) — supporting native coverage. Focused no-login probe (`/tmp/uar111-r5-r11-noauth-probe.py`): the same 401-on-no-auth pattern for `/api/uar/agents`, `/api/uar/sessions/.../agent-config`, `/api/uar/sessions/.../effective-config`. The contract’s "no-login REST create/configure" path is blocked by the same product gap. The MCP-server piece needs `tests/mcp_child_environment.rs` (requires `test-probes`, out of scope). **Next**: run `mcp_child_environment` when its feature is allowed. |
| R12 | UNVERIFIED | No model turn executed. **Next**: two consecutive harness POSTs (config + agent); assert the 2nd routes through the new provider/model. |
| R13 | UNVERIFIED | No model turn executed. **Next**: model turn with a chosen modality set + explicit capability-table assertion. |
| R14 | UNVERIFIED | `tests/builtin_db_registration.rs` COMPILE ERROR (pre-existing — not in redirect scope to fix). **Next**: representative flow against embedded SurrealKV via the existing acceptance harness (not via the broken native test). |
| R15 | UNVERIFIED | No harness run. **Next**: repeat the R14 representative flow against remote SurrealDB. |
| R16 | UNVERIFIED | No harness run. **Next**: harness KB create/list + upload/index. |
| R17 | UNVERIFIED | No harness run. **Next**: vector-search call after R16. |
| R18 | UNVERIFIED | No model turn executed. **Next**: model turn that requires KB content; capture the source evidence in the reply. |
| R19 | UNVERIFIED | No harness run. **Next**: delete a doc; assert direct search returns no hit + a fresh-agent retrieval returns no hit; then delete the KB. |
| R20 | PARTIALLY GREEN | `in_memory_spawn_persists_identically_ordered_threads_and_edges` PASS (in-memory persistence + ordered threads/edges). The "no resurrection across reconnect" half + the Surreal variant is UNVERIFIED. **Next**: harness restart probe against the same Surreal volume. |
| R21 | PARTIALLY GREEN | `tests/context_history_integrity.rs` 20 of 33 ok + 2 ignored (frozen-source diagnostics) before SIGABRT in `run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch` (a run_manager product issue, not a context-trim policy issue). The 20 ok tests demonstrate the trim/summary behaviour for budget + protected content + overflow paths. The run_manager resume/persist tool-call path is a separate product issue (UNVERIFIED for that sub-aspect; the crash indicates unbounded recursion in the run manager, fixable in a repair cycle if scope is allowed). The full "fits real model window + reserved output" assertion remains UNVERIFIED. **Next**: harness turn with deliberately oversized input + captured assembled-input (model needed). |
| R22 | PARTIALLY GREEN | `tree_limits_refuse_the_fifth_concurrent_child`, `*_the_fourth_child_nesting_level`, `*_the_seventeenth_lifetime_child` PASS — refusal semantics verified at multiple budget classes. The "unknown-liability refusal" + exact "refuse BEFORE the action" trace is UNVERIFIED. **Next**: model-side reasoning trace under a budget-bound input. |
| R23 | PARTIALLY GREEN | `*_spawn_persists_identically_ordered_threads_and_edges` (in_memory + surreal) PASS (ordered threads/edges). The "joined results + cancellation no-orphan" trace is UNVERIFIED. **Next**: model run + explicit cancellation; assert no new side effects. |

**Status totals (truthful — no 0-FAIL claim)**:
**0 contract-level PASS · 11 PARTIALLY GREEN · 11 UNVERIFIED · 9 observed failures / gaps preserved separately** (A2A 3/4, dot_memory_owner 2/2, context_history_integrity SIGABRT, builtin_db compile, bauar_resource_grants 1/8, **R6 product gap — route requires auth; no-login blocked; model unverified**, **R7 product gap — route requires auth; phase-user principal lacks Anthropic source authority**, **R9 product gap — AG-UI route requires auth; no-login 401; with-JWT 200 + RUN_ERROR at credential_binding is only the authenticated transport subpart**, **R5/R11 product gap — REST surface (4 endpoints) returns 401 without an Authorization header; no no-login REST mode is configured**).

**Repair cycles**: 2 of 4 consumed (round-6 cycle 1 = Surreal replay fallback + fixture capture; cycle 2 = type-path correction). 2 remaining. The three adjacent product issues (`run_manager` stack overflow; `dot_memory_owner_contract` owner-boundary failure; `bauar_resource_grants` revision-lease failure) are recorded but not consumed from the cycle pool — a repair cycle would only be used if the operator authorizes a within-scope fix and a contract sub-criterion is gated on that fix. The R6/R7/R9 product/config gaps are recorded as adjacent observations; no admin authority was added, no scope expansion, no silent service-account injection.

## Cumulative accounting (KBD-tracked; preserved across the redirect)

- **Before continuation** (round-6 close at 4,447 s / ~28 tool calls / 2
  repair cycles; goal close 21:02:26 UTC).
- **Operator-authorized additive** (2026-10-18): +90 min, +60 tool calls,
  +2 repair cycles.
- **New total ceiling**: 240 min / 130 tool calls / 4 repair cycles.
- **This continuation** (operator "Continue the existing approved
  allowance ... this turn ended with the frozen contract incomplete"):
  added wall time for the 4 extra native test runs (bauar_resource_grants,
  bauar_secret_projection, bauar_identity_boundary, agent_policy_intersection),
  the R6+R7 with/without JWT live demo, the R9 no-login live demo, the
  R9 with-JWT live demo, plus the row/report file edits and reconciliation.
  Best-estimate added tool calls: ~20-25 (the operator can verify the
  exact count from the transcript). No silent time / call / cycle
  extension. No silent service-account injection. No admin authority
  added.
- **Estimated residual under the new ceiling** (using the rough
  estimate): ~155 min wall, ~77 tool calls, 2 repair cycles. The
  previous "comfortable" framing is replaced by this numerical estimate.
- No commit, push, tag, release, or deploy.
