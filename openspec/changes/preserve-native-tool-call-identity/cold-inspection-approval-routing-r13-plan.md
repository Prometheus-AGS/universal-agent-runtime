# Cold inspection, process-resident approval routing and full r13 backend gate

Stage: Execute. Assessment and analysis are complete in the retained
cold-run-inspection-assessment-r13.md, including its superseding follow-up.
Verification is CLOSED after r12 first failure; no earlier instruction covers
this source batch or its future execution.

## Observed problems and chosen behavior

R12 passed its source/model/predecessor guards and both initial runtime startups,
then fresh_process_resume failed: the cold owner's run GET expected404 and got200
with actual durable done metadata. Both the fallback and stale assertion exist in
baseline774469746fee4de99be41dc1cd231857067e4487. Preserve durable inspection.
Source assessment also confirms a separate production defect: both public
approval handlers use this durable-inclusive lookup to choose a live process
branch. A history row cannot recreate the broker's absent root/waiter. The
existing pending-restart and cold decision assertions are correct; fix routing.
R12 stopped2, target0/1, six owned backend resources removed, no source/evidence
failure. Stream/approval/build never started. Preserve every receipt and source.

## Complete production/fixture batch and one-writer ownership

The trust-tools writer owns exactly four paths:

- src/uar/runtime/manager.rs: add a crate-private authenticated process-resident
  run predicate/lookup, reproducing the exact existing active-branch owner,
  subject/tenant and anonymous consistency checks. No durable load, mutation,
  authority grant or new API. Preserve get_run_for_context entirely, especially
  its immediate rejection for a wrong-owner resident entry. Retained terminal
  entries are still process-resident; do not substitute waiter presence.
- src/uar/api/routes.rs: change ONLY the two public approval live/cold branch
  conditions to the authenticated process-resident capability. Preserve response
  schemas, principal/workspace checks, approval CAS, expiry/legacy/deny behavior,
  checkpoint admission/resource renewal, claims and all errors. No global caller
  replacement and no changes to history/read_run or host-specific semantics.
- tests/support/fresh_process_resume_process.rs: extend existing joined readback
  child to return actual optional RunHistoryRecord and ordered RunHistoryEvents,
  preserving its checkpoint/envelope/claim/epoch checks and shared callers.
  Build exact ten-field RunInspection expectation from that captured record;
  independently absent abrupt history alone permits404, otherwise require200
  with exact saved public fields and owner/run binding. Orderly normal records
  must exist with done; uncertain error derives from the actual drained record.
  Capture provider/effect and active-list state around repeat owner GET and
  checkpoint listing, prove no changes, unknown run404 and existing negative
  subject/tenant/issuer GET404/404/401. Target verification stays outside this
  read-only interval. In each normal resume local/shared case, join an owned
  inspection-only cold node, perform independent readback with zero claims and
  equal source history/events/envelopes/epoch, then boot a distinctly labelled
  execution node and retain all original resume assertions and PID distinctions.
  No concurrent embedded KV reader, extra abrupt/uncertain shutdown, new coverage
  row, changed deadline, weakened policy/claim/effect assertion or new general
  history immutability promise. Full conservation is ONLY inspection-only.
- tests/support/approval_recovery_process.rs: strengthen existing orderly pending
  restart scenario with authenticated live inspection before shutdown and exact
  cold owner GET200/source identity before its existing pending and decision
  assertions. Preserve all owner/tenant/issuer negatives, live/cold behavior,
  resolvable/recoveryAvailable distinction, deny/expiry/legacy/policy/binding/
  race/unknown/sibling canonical receipts. Existing real HTTP recovery cases are
  the regression oracle; no new unit-only gate or integration target is needed.

The writer may write its handoff only under the native identity handoff root.
Others share the checkout: preserve their changes. No producer path outside this
list is granted; stop and report any additional causal scope before editing it.

The gate writer owns ONLY new scripts/release/native_identity_backend_gate_r13.py.
Reuse hash-bound unchanged baseline supervisor and support module directly; do
not import/install the prefix-adopting continuation adapters. Run the baseline
FOUR complete targets serially, then its exact customer build, with no additional
unit/protocol/identity/client rerun or arbitrary target selector. Production routing
changes invalidate the prior prefix continuity proof. Historical10/10 is retained
as history, never adopted as current pass. Bind exact r12 predecessor receipts,
source/settings references for preservation, but claim no acceptance from them.
Keep six shipped asset identities/modes/blob bindings, external Cargo controls,
full-source/index/submodule checks, settings protection, existing backend image/
platform/authentication checks and owned cleanup/evidence rules. Recheck immutable
inputs at every source capture. Do not change r10/r11/r12 script bytes or bypass
any validator. New fixed source/settings/output names: source-manifest-r13.json,
backend-settings-r13.json, backend-evidence-r4, backend-resources-r4 under the
existing private parent. No credentials on argv or in public records.

The lead owns this plan, the existing spec delta/task/design/operational docs,
private settings, source integration, complete inventory and KBD/board records.
Add the narrow durable-inspection/process-resident dispatch scenario to existing
approval spec and operational docs; keep r12 history unchanged. Existing47
producer paths plus routes and new plan/driver yield50 producer paths before any
justified extra documentation path (existing docs change in place). All27 controls
remain unchanged, including six existing model/notice files; no new dependency,
schema, catalog, toolchain, model, backend or public API change is planned.

## Completion, reviews, refreeze and later exact approval

Finish all writer handoffs, required docs/config/runtime inputs and source-only
integration. Capture explicit preimages/delta/full source/controls and exact tree;
stop writers and reconcile the complete scoped canonical task ledger. Obtain
separate artifact-only product and security reviews after implementation, with
one corrective pass at most and explicit screening/family limits. Source review
proves no runtime result. Only after all scoped source is complete and integrated
ask for the exact single new wrapper invocation. All tests/builds/format/lint/
compiler/import/syntax checks/validators/model/runtime probes remain held until
that explicit approval; no incidental preflight or automatic compiler exception.

The proposed future invocation is python3
scripts/release/native_identity_backend_gate_r13.py --settings
/Volumes/T7/pr13-native-identity-r9-private/backend-settings-r13.json. Its embedded
source/retained-input/model guards and fresh owned backend setup are in scope.
Stop on first failure, missing coverage or drift and preserve failed evidence;
attempt cleanup only for owned resources. No overall finite watchdog is claimed.

The uncomfortable limit: this production branch correction changes source under
the earlier passing prefix, so rerunning all four targets is necessary. None is
currently certified on r13, and source inspection cannot promise the remaining
scenarios pass. Native inference, formatting/hooks, warnings, ignored provisioning,
broader certification, actual immutable publication and Kubo/Desktop consumer
integration/cumulative verification remain separate holds. R8/protocol11 evidence
remains historical and untouched; PR13 stays draft, with no ready/merge claim.
