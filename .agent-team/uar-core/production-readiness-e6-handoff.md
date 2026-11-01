# E6 source repair handoff — 2026-11-01

Phase: `production-readiness-476-capability-dispatch`, Execute.
Verification gate: **CLOSED_OPERATOR_LOCK**. No verification was run or delegated.

## Owned source change

`src/uar/runtime/cost_budget.rs`, existing test
`durable_contract_without_final_wire_evidence_refuses_before_inner_stream`:
replaced `stream(contract_request).await.expect_err(...)` with an explicit
`match`. Unexpected `Ok(_)` retains the exact fixed panic message; `Err(error)`
extracts the error without requiring `Debug` on the successful
`ExternalDriverStream` type. The existing admission-origin, root-cause text,
and zero-inner-dispatch assertions remain unchanged.

The observed E6 scenario is the test's use of `expect_err`, whose successful
stream type lacks the required `Debug` implementation. No public stream
contract, production behavior, dependency, guard, or unrelated code changed.
No unrequested implementation was added. Existing rc.2 integration and E work
were preserved through a minimal replacement of the assigned assertion.

## Commands and evidence limits

Only source reads and authorized file writes were performed: `pwd`, `sed`,
`cat`, `rg`, `tail`, `ls`, and the patch application. The initial `ls` reported
that this handoff file did not yet exist. Source inspection located the
reported `expect_err` expression and the three retained assertions. No Cargo,
test, build, format, lint, compiler probe, API/container operation,
`git diff --check`, or review gate ran. No tests were delegated.

Compilation, warning cleanliness, and runtime behavior remain unverified;
the operator lock prohibits their verification. This handoff is source-only
implementation evidence and earns no acceptance credit. There are no
unresolved decisions within E6. The lead must include this source repair in
the renewed whole-phase inventory before proposing exact gate commands for
operator approval.
