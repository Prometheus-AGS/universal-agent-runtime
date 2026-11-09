# E2 runtime source handoff

Phase: `production-readiness-476-capability-dispatch`, Execute. Verification is
**CLOSED**. This handoff records source inspection and authorized documentation
only. No test, lint, format, typecheck, Cargo command, build, API call, or
review-driven verification was run.

## Disposition

E2 is a stale fixture-admission/durability setup, not a production runtime
defect found in the owned runtime surface. No runtime source file was changed.

The historical failing test constructed `RunManager` with `persistence: None`
and called `start_run` with `user_id: None`. That exercises the anonymous,
non-durable compatibility path, while E2 requires an admitted owned run with a
durable pending assistant call.

The source already contains the required guarded behavior:

- `src/uar/runtime/turn/request.rs:338-353` accepts a `UserContext` only by
  deriving `ActorOwner` through `ActorOwner::from_verified_context`; fixture
  input cannot manufacture a verified owner.
- `src/uar/runtime/manager.rs:4720-4778` chooses the verified owner's session
  storage key, restores only within that partition, and rejects a non-anonymous
  unverified conversation owner.
- `src/uar/runtime/manager.rs:9896-9914` records an unfinished assistant tool
  call in both run dialogue and retained session, then durably saves the retained
  session when persistence exists.
- `src/uar/runtime/manager.rs:9986-10005` leaves the run in `Error` and records
  the pending call identifiers. `src/uar/runtime/context/reduce.rs:425-429,487-491`
  rejects a history with a missing tool result; manager assembly converts that
  rejection to an error and returns before provider execution at
  `src/uar/runtime/manager.rs:7833-7869` and `7928-7966`.
- `src/uar/runtime/checkpoint_host.rs:55-61` remains unchanged: anonymous runs
  have no resume authority and cannot mint resumable checkpoint authority.

This preserves verified identity, owner partitioning, durable-mutation ownership,
approval/uncertain-effect controls, and the existing resume-dispatch refusal.

## Fixture handoff accepted by the fixture owner

`tests/context_history_integrity.rs` is owned by the fixture writer. The writer
implemented the recommended E2 setup without runtime changes:

- fresh temporary `SurrealDbProvider` supplied to `RunManager`;
- production-verified `UserContext`, produced through the existing verifier
  bootstrap before the fixture signer;
- `RunExecutionRequest::with_user_context(&user)` plus `execute_request` for
  both turns;
- owner-scoped `SessionStore::get_for_user` and durable `load_session` equality
  assertion; and
- retained first-error, pending-call, `MissingResult`, second-error, and one
  provider-dispatch assertions.

`tests/support/harness_context.rs` now holds the shared verifier bootstrap. It
is test-owner work, not a runtime interface change.

## Changed paths owned by this runtime task

- `src/uar/runtime/cost_budget.rs` (E1b durable reservation integration)
- `src/uar/runtime/thread/service_tests.rs` (E1b held-stream durable consumer
  fixture)
- `.agent-team/uar-core/production-readiness-context-runtime-handoff.md` (E2
  disposition and E1b source handoff)

E2 changed no runtime behavior. E1b changes the private budgeted-driver
interaction: a durable request with a final-wire contract now requires the
captured driver to return `PreparedFinalWireBudget`; no-durable and no-contract
paths retain their existing behavior. The provider-owned trait and Liter
implementation are recorded separately by that owner.

## Evidence and limits

The historical failure evidence is
`.kbd-orchestrator/phases/production-readiness-476-capability-dispatch/evidence/context-history-attempt-001.*`:
`run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch` failed at
the absent pending-call assertion. It proves the old fixture failed, not the
cause of a production defect.

Compass graph discovery was unavailable in this worktree because
`compass-out/graph.json` is absent. The source anchors above are therefore
direct-source evidence, not graph/runtime proof. The fixture changes and the
existing runtime path remain unverified until the lead completes the whole-phase
inventory and obtains operator authority for the named gate.

## E1b source-only reservation-coherence design trace

E1b was added to the same Execute cohort. Verification remains **CLOSED** and
this handoff records the implemented narrow source change only; no verification
command ran.

### Observed representation mismatch

- `src/uar/runtime/cost_budget.rs:1706-1735` narrows the request, performs
  tracker admission, then calls `ModelCallBudget::begin_call` **before**
  `inner.stream(request)` reaches the captured provider driver.
- `cost_budget.rs:2166-2185` currently obtains its durable input reservation
  from `liter_driver::request_budget_bound` and labels it
  `final_wire_contract`. That helper serializes a UAR preflight representation
  (`src/llm/liter_driver.rs:743-784`), not the captured Liter client's bytes.
- `LiterLlmDriver::stream` builds the request, then immediately before
  `chat_stream` obtains `client.chat_stream_request_body(&prepared.request)`,
  validates it, and counts that actual prepared body
  (`liter_driver.rs:858-905`). Its existing `profiled_wire_receipt`
  (`liter_driver.rs:204-241`) already demonstrates the same captured-client
  preparation path for profiled requests.

Therefore the current durable reservation cannot truthfully claim its UAR
preflight count is an exact `final_wire_contract`; source does not prove either
representation bounds the other.

### Agreed narrow seam

The provider owner added the crate-private, object-safe
`LlmDriver::prepared_final_wire_budget(&LlmRequest) -> Result<PreparedFinalWireBudget>`
in `src/llm/mod.rs`. Its default refuses a budget-contract request, so no mock,
external, model-name, or UAR-preflight fallback can certify final-wire input.
`LiterLlmDriver` implements it in `src/llm/liter_driver.rs` by sharing its
captured-client request preparation and `chat_stream_request_body` path with
the leaf validation. The returned value contains actual serialized input tokens
and the effective selected output ceiling; approximate counting and unsupported
contract states refuse before dispatch.

The owned runtime integration is limited to
`src/uar/runtime/cost_budget.rs`: after `narrow_team_output_cap` and before
`begin_call`, `BudgetedModelDriver` requests this bound from its captured inner
driver. `begin_call` receives it, adds only existing contract input uncertainty
and additional reasoning output with checked arithmetic, and retains the
durable `admit_accounting_call` path. The provenance label becomes
`liter_prepared_final_wire_contract`, replacing the inaccurate
`final_wire_contract` label.

This preserves atomic durable reservation before provider dispatch, finite input
liability, output ceilings, cache-conservative cost reservation, owner scopes,
replay/uncertain settlement, cancellation, and root-deadline behavior. It does
not create a second accounting path or relax an admission limit.

### Captured-chain and unsupported limits

`RunModelBindings` places `BudgetedModelDriver` around the captured provider
driver (`src/uar/runtime/turn/bindings.rs:392-396,414-418,583-595`). For graph
sends, outer `DestinationBudgetDriver` installs/narrows the contract before
this wrapper (`src/uar/runtime/manager.rs:77-109,8244-8251`). Hierarchical
summary wrappers likewise modify and prepare their request before dispatching
to the governed inner driver (`src/uar/context/strategy.rs:438-466`). These
observed paths make the proposed timing source-supported.

`ExternalLlmDriver` already refuses a final-wire contract
(`src/llm/external_driver.rs:59-65`). Mock and test-only drivers may carry a
contract in fixture assertions but cannot prove a Liter-prepared body; when
routed through durable `BudgetedModelDriver`, they must refuse unless a distinct
explicit deterministic capability is later designed. No production fallback or
model-name exception is proposed.

### Authored assertion inventory and fixture disposition

Provider ownership added, but did not run,
`src/llm/liter_driver.rs::prompt_caching_tests::prepared_final_wire_budget_uses_liter_bytes_and_refuses_image_liability`.
It independently observes the captured Liter-prepared bytes, checks their input
count and effective output ceiling, and refuses an image-bearing request so
serialized JSON cannot certify real-image liability. Fixture ownership also
retains the real-wire assertion
`tests/context_history_integrity.rs::exact_budget_contract_dispatches_intact_protected_wire_body`.

No new durable-accounting fixture was authored: the existing
`src/uar/runtime/thread/service_tests.rs::Harness::start` now retains its
explicit held-stream lifecycle while delegating only
`prepared_final_wire_budget` to a captured `LiterLlmDriver` built from the
same `LlmConfig`. It derives the expected pending durable reservation from that
actual bound instead of the removed static preflight consumer. The existing
bounded consumer/refusal scenarios
`remote_admission_refusal_does_not_lease_root_budget`,
`remote_persisted_launch_refusal_releases_capacity_and_shutdown_settles_record`,
and `remote_pending_cancellation_releases_capacity_before_any_dispatch` retain
their real Surreal accounting collaborators and now share this actual-prepared
reservation assertion. The held fake stream is never treated as final-wire
evidence.

The runtime task also adds, but does not run,
`src/uar/runtime/cost_budget.rs::tests::durable_contract_without_final_wire_evidence_refuses_before_inner_stream`.
It is explicitly synthetic seam coverage: it marks the budget durable only to
reach the contracted admission branch, uses the existing in-memory accounting
collaborator, and proves an unsupported prepared-bound driver returns the
admission failure before entering its fake stream. It does not certify durable
storage, provider transport, actual Liter bytes, or image capacity; the real
Liter assertion above covers prepared bytes and image refusal, while the held
thread fixture covers the actual durable consumer reservation.

The provider owner removed the obsolete static `request_budget_bound` helper
after this adaptation. `preflight_budgeted_request` remains auxiliary-only;
there is no static preflight route into durable reservation. This is source
state, not a verification result.

The design and all assertions remain unverified until the lead completes the
whole E1--E5/E1b source inventory and invokes the phase's existing explicit
bounded acceptance authorization.
