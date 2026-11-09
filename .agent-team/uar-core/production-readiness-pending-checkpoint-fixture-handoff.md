# Pending history / checkpoint fixture handoff

Phase: `production-readiness-476-capability-dispatch`, Execute.
Verification: **CLOSED_OPERATOR_LOCK**. Every command in this task was source
inspection or authorized fixture/document writing. No test, check, lint,
formatter, compiler, build, review gate, API, container command, commit or hook
was run. All edits below are **UNVERIFIED**.

Owned files:

- `tests/context_history_integrity.rs`
- `tests/support/harness_context.rs`
- this handoff

The checkout is shared. Other writers' edits and configuration were preserved.
The reported failing gate source was `b3e5eda9729daf2a160d75aaea04f43486dd8f40`;
the lead owns the renewed exact source identity and whole-phase inventory.

## Source findings and selected repair

`RunManager::new` constructs an empty `NativeSkillRegistry` (manager.rs
1344–1353). Consequently, the initial native-descriptor preselection snapshot
(6136–6140) does **not** prove a hard-capability refusal. An earlier hypothesis
that it did was explicitly corrected before implementation.

The pending fixture does project a later tool requirement: `activate_skill` is
registered at manager.rs 6305–6317 because its permission follows Auto skills
(`domain/policy.rs`, `allows_model_control_tool`), independently of the empty
initial native snapshot. The actual request's final destination admission
requires registry metadata when tools are present (`llm/router.rs`,
`DestinationAdmission::admit`, `None if has_hard_capability_requirement`). Both
existing fixtures omitted a provider registry; the pending fixture therefore
cannot reach its synthetic stream through this final capability seam.

For that observed pending-fixture mismatch, `harness_context.rs` now supplies an
actual `ProviderRegistry` containing explicitly enabled `openai/gpt-4o`, tools
and streaming capability. The pending fixture installs it and explicitly selects
the same artifact provider/model. An exact artifact route is necessary because
registry installation also activates per-artifact provider resolution; leaving
the artifact's provider fields blank would introduce another admission refusal.
The stream remains the existing explicitly synthetic `MockLlmDriver`.

No exact final-wire capability was invented. These fixtures install no
destination preparations; manager.rs 7341 derives no request budget contract.
`cost_budget.rs` 1729 only requests captured final-wire proof for durable calls
with a request budget contract, so these existing fixtures retain their real
catalog-based durable accounting lane. Verified middleware identity, temporary
SurrealKV persistence and owner-qualified history loads remain intact.

There is a second, independent pending-runtime seam: orchestrator.rs accumulates
`ToolCallComplete` privately and suppresses public completion until EOF/batch
validation; a subsequent normalized Error returns before that completion is
published. The fixture's original Complete-then-Error event sequence is unchanged.
The lead assigned the runtime teammate a native-only incremental observation
repair that must preserve full-batch completion and caller-tool gating.

## Fixture changes

`run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch` now requires
RunStart, the exact synthetic `PROVIDER_INTERRUPTED` error/message, and exactly
one initial provider call before looking for the pending assistant call. Resume
must produce `invalid_history` naming `pending-1`, in addition to all existing
empty assistant, MissingResult, owner-qualified durable complete-history equality,
second Error and exactly-one-provider-request assertions.

`checkpoint_resume_rejects_raw_metadata_and_legacy_admission_without_mutating_history`
retains its three-second graph capture bound. Its waiter also notices an observed
Error/Cancelled admission instead of awaiting only the channel. Failure now
includes actual run state, event history, execution count and provider-call count.
Successful capture additionally requires one graph-node execution and zero
provider calls. Existing raw-metadata 404, revoked 404, direct-legacy error,
execution/provider equality and warm-history equality assertions are unchanged.

## Open checkpoint diagnosis

The checkpoint timeout is **not repaired or explained** by the pending registry
repair. No registry was added to that fixture speculatively. Source tracing found
that graph admission starts through GraphRootSupervisor and ActorThreadSession;
the root has Running status, verified subject and durable persistence; model
backfill supplies the configured openai/gpt-4o route; captured model binding and
root-policy credential identity agree. The graph engine invokes its capture node
before any provider request and the capture node finishes without automatic
checkpoint persistence. Neither the retained gate receipt (only panic/status
excerpts) nor the inspected source establishes the actual failed invariant.

The bounded diagnostic assertion is an evidence improvement, not a production
fix. The lead/runtime teammate must resolve the graph cause or explicitly retain
it as unfinished implementation before asking for any gate approval. Do not
increase its timeout, substitute identity/persistence, or claim acceptance from
source inspection.

## Remaining work / verification

- Runtime owner completes its assigned pending partial-observation repair.
- Graph source diagnosis remains open; no justified graph behavior edit selected.
- All authored Rust, stage assertions, persistence behavior, denial behavior,
  warning-free compilation and lifecycle behavior remain UNVERIFIED under the
  operator lock. Source inspection only was used; no passing result is claimed.
- No unrequested feature or new guard was added. Added assertions trace to the
  observed unrelated Error acceptance and uninformative capture timeout.
