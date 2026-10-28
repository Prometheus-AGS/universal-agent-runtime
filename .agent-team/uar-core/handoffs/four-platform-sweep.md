# Task 2.21 — prerequisite-aware serial sweep handoff

Date: 2026-10-25. Phase: `uar-post-lifecycle-verification-readiness`,
`user-outcome-implementation`. Owner: acceptance_sweep teammate; lead retains
canonical waypoint/task state and the whole-phase implementation inventory.

**Operator verification gate CLOSED.** Only source inspection and authorized
source/test/document authoring occurred. No imports, tests, lint, formatting,
syntax/compiler probes, builds, service calls, provider calls, source-identity
probes, review gates, or external actions ran. This handoff grants no execution
permission and does not open the gate. Other teammates share this checkout;
preserve their edits. No writer's source identity was frozen by this teammate.
The lead must capture fresh exact identity after all writers finish.

## Files delivered

- `scripts/acceptance/user_outcomes.py`: optional `collect_results=False` API
  parameter and explicit `--collect-results` CLI flag; compatibility default
  remains first-failure. Opt-in refuses an unfrozen controller module.
- `scripts/acceptance/failure_sweep.py`: separate full serial controller; fixed
  97 × 3 × 2 inventory, explicit earlier-case prerequisites per backend/repetition,
  fresh isolated Context and admission/confirmed cleanup per attempt, one live
  packet budget owner, durable attempt/failure/unresolved receipts and aggregate.
- `scripts/acceptance/failure_sweep.README.md`: public fixture, approval, inbox,
  receipt, correction refresh, budget, evidence, exit-code and isolation contract.
- `scripts/acceptance/tests/test_failure_sweep.py`: authored UNRUN tests for
  denominator, forward prerequisites, same backend/repetition dependencies,
  independent continuation, no implicit retry, unknown cleanup stop, mixed source
  refusal, one explicitly admitted corrective rerun and wrong-sweep receipt refusal.
- This handoff.

No edits to core.py, packet_processes.py, compose_fixture.py, source_inventory.py,
case/helper modules, phase ledger, OpenSpec or other teammates' owned files.

## Public opt-in semantics / composer follow-through

Exact future command keeps the runner's `--execute --fixture <absolute fixture>
--evidence <new absolute evidence directory>` and adds `--collect-results`.
No command was executed. `--case` is rejected in collection mode to preserve
all582 planned rows. First-failure mode keeps its existing subset support.

Fixture must freeze failure_sweep.py and supply:
`failure_sweep: {schema_version:1, prerequisites:{<every97caseID>:[earliercaseIDs]}}`.
Composer owner must add that contract to its source; no fixture or authority
was generated here. Source inspection found cases individually configure their
provider and own conversations/resources: U02 does not consume U01's session,
U07 does not consume U06's specialist fixtures, U05 and shipped UI cases each
perform their own configuration. All may explicitly start with empty case lists;
actual setup/provenance invariants inside helper methods remain mandatory.
Do not manufacture a blanket dependency on U01's useful-answer predicate.
Additional true case dependencies can be declared explicitly in manifest order.
Packet and per-item backend admission prerequisites are recorded separately.

Optional frozen correction_authority `{path,sha256}` identifies actual externally
adopted bounded operator approval. Fields: schema_version1, operator_approvedtrue,
authority_id, initial source_identity/artifact_identity, current epoch_id,
max_corrections1..582, wait_seconds>0..3600, private receipt_directory, nonempty
refreshable_paths naming existing frozen paths. Initial source+artifact binding
avoids a circular fixture-digest/authority-file reference. Exact gate approval
must bind approval bytes and authorize the trusted host correction effects.
A JSON flag or self seal does not itself grant permission.

Default has no authority and never waits/retries. With authority, FAIL records
its exact receipt then pauses only within shared wall bounds. Trusted host
performs separately authorized targeted correction and artifact refresh, then
atomically publishes the read-only sealed item receipt specified in README.
The controller executes no correction command/agent/build. Receipt binds exact
sweep/item/failure/old+new identities, refreshed fixture hash and one correction.
It then admits new fixture and reruns this item once in fresh state. Corrective
failure is appended and unresolved; no second fix/rerun. Missing receipt logs
unresolved and continues independent items after cleanup. Invalid receipt stops
packet conservatively because source may already have changed.

The same packet process remains alive through bounded waits, so epoch counters,
spend reservations and wall time are never reset by a correction. No cross-run
resume is implemented. A later invocation requires separately approved epoch
adoption with reconciled prior usage; this controller never creates that authority.

Refresh keeps argv, input path sets, limits, provider/backend bindings, schema
revision and dependency contract identical; only source/artifact/configuration
revision and explicitly authorized existing frozen hashes may change. Python/MJS
helpers, live packet argv files and authority remain immutable. A helper/controller
repair, newly named artifact, schema change or altered allowance requires a fresh
approved packet. Derived observer JSON metadata may refresh at explicitly approved
paths so new source/artifact admission remains possible. New passing source cannot
reuse earlier passing dependency rows from old source. Mixed-source successful
rows cannot produce aggregate PASS; they require coherent full rerun.

## Evidence and limitations

Existing Evidence exclusive fsync/hash-chain writer records full inventory,
initial/corrective attempts, terminal/failure receipts, correction request and
consumption, refresh identity, unresolved rows, safety stops and aggregate.
Historical failures stay in receipt/result history. Latest admitted terminal
status occupies its original inventory row. Non-run rows remain in denominator.
Exit0 PASS,1 unresolved FAIL,2 BLOCKED/safety-stop. phase_acceptance alwaysfalse.

Cleanup ownership unknown or exhausted/unknown shared budget stops all independent
dispatch because ports/host and epoch are shared. Failed backend admission blocks
remaining cases on that backend after its cleanup; the independent backend may
continue. Every new attempt starts empty store/server, which is deliberately more
expensive than backend-reused compatibility execution and needs explicit budgets.

All runtime behavior, syntax, test expectations and customer deliverability remain
UNVERIFIED under CLOSED. No source inspection claim is an acceptance receipt.
Independent review and exact planned production-path gates await the lead's full
implementation inventory and new explicit operator instruction. No review agent
or verification command was dispatched. No speculative guards: receipt/file
ownership and frozen identity checks guard the actual host-to-runner authority
boundary; fixed denominator, dependency isolation, mixed-source refusal, resource
bounds and cleanup stops trace directly to task2.21/user continuation requirements.
No unrequested production changes were added.

## Lead-requested shared owner follow-up

Source-only API extension: execute_collection now accepts optional `packet_owner`
and `shared_state`. A borrowed owner requires shared state and is never created,
started, stopped, or re-observed by this module. The wrapper owns one live
PacketProcesses for all platforms and its final cleanup. Per-item owned cleanup
remains mandatory. Standalone default ownership remains unchanged.

Shared dictionary contract is started (original monotonic suite start), requests,
output_bytes, corrections, max_corrections. Counters are adopted at entry and
persisted after each item cleanup, including failures. Local correction authority
and shared cap both bound correction dispatch; shared correction consumption is
incremented before artifact refresh and same-item rerun, never reset. Exhausted
correction slots leave unresolved failures while independent work may continue.
Shared resource/ownership safety stops require the wrapper to stop further
platform dispatch. The wrapper must admit compatible fixture/source/proxy inputs.

Summary adds borrowed_packet_owner, shared_corrections_consumed,
final_source_identity, final_artifact_identity, final_fixture_path and
final_fixture_digest so the lead wrapper can reconcile source continuity.
Authored two additional UNRUN policy tests: borrowed owner has no start/stop and
preserves initial counters, and an exhausted12-correction suite cap cannot reset
at a later platform. No verification command ran; operator gate remains CLOSED.
