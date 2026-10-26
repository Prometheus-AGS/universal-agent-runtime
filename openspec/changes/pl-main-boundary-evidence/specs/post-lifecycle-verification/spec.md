## ADDED Requirements

### Requirement: Boundary execution requires a complete candidate-bound packet and explicit authority

The finite boundary packet SHALL bind the complete implementation inventory, source and lock/toolchain inputs, effective isolated configuration, fixtures, adopted contract and exact process and child-command arguments before execution. Source authorship, publication success and historical permissions SHALL NOT substitute for the operator's exact current verification and fixture authority.

#### Scenario: Packet remains proposed

- **WHEN** implementation inputs, the applicable contract adoption or exact verification and fixture approvals are incomplete
- **THEN** the packet remains non-runnable and no successful publication or checked authoring task opens product verification

#### Scenario: Candidate identity differs

- **WHEN** the approved source or fixture inputs change, or the booted main executable differs from the bound built candidate
- **THEN** the boundary cannot report PASS and retains the actual identity mismatch for renewed inventory and affected authorization

### Requirement: Finite evidence retains every scenario and its actual scope

The main boundary SHALL retain LIAB-01, LIAB-02, ERROR-01, ERROR-02, COUNT-01, HISTORY-01, KB-01, CAS-01, BTHREE-01 and IDENTITY-01. It SHALL distinguish PASS, FAIL, BLOCKED and NOT_EXERCISED. Missing scenario steps or required controls SHALL NOT create synthetic success. COUNT-01 SHALL retain its trusted-builder scope and standalone-main UNSUPPORTED label; absent B3 evidence SHALL retain runtime NOT_EXERCISED.

#### Scenario: Required controls are absent

- **WHEN** an included behavior scenario lacks reviewed steps, required controls or successful setup
- **THEN** its receipt remains BLOCKED and the missing evidence is not converted to a passing result

#### Scenario: Proof scope is limited

- **WHEN** trusted-builder COUNT evidence or the absent-original-archive B3 disposition is recorded
- **THEN** each receipt preserves its distinct evidence scope without asserting standalone-main exact-fit support or a B3 runtime PASS

### Requirement: Owned teardown and append-only receipts are part of the outcome

The runner SHALL retain separate setup, body and teardown outcomes in append-only evidence. It SHALL signal only declared owned children, await their actual exits and remove only explicitly authorized exact owned fixture resources. A passing body SHALL NOT hide failed cleanup, unresolved ownership or teardown failure.

#### Scenario: Body passes but teardown fails

- **WHEN** the behavior cases pass but an owned child exit, row or schema-event cleanup remains failed or unknown
- **THEN** the overall result is not PASS and the receipt retains the failed teardown or outstanding obligation

#### Scenario: Existing evidence destination is reused

- **WHEN** execution attempts to write a receipt at an already occupied append-only evidence destination
- **THEN** the runner refuses replacement rather than overwriting the predecessor's outcome

### Requirement: Startup readiness and cleanup retain stage ownership

The fixture driver SHALL retry connection transport failures only for explicitly selected startup GET readiness polls against an owned child, within their existing attempt and sleep bounds. It SHALL retain each failed attempt and refuse an observed exited child. Ordinary scenario transport errors SHALL remain failures. Peer output files SHALL belong to the current append-only attempt. Cleanup SHALL distinguish resources this attempt could not create from failed cleanup after a main process was launched; this distinction SHALL NOT turn failed setup, missing process counts or a nonzero or unconfirmed child exit into PASS.

#### Scenario: Owned child has not begun listening

- **WHEN** an explicitly selected startup readiness GET has a connection transport failure while its owned child remains running
- **THEN** the driver records the attempt and uses only the already declared bounded polling attempts, without starting a second child or retrying a scenario operation

#### Scenario: Main never launched

- **WHEN** setup fails before this attempt ever launches main
- **THEN** main HTTP cleanup operations are recorded NOT_APPLICABLE with the main-never-launched reason rather than issuing deletes for resources this attempt could not create, while setup/scenario/process-count failures remain blocking

#### Scenario: Main launched or child exit fails

- **WHEN** main was launched at any point, or an owned child returns a nonzero or unconfirmed exit
- **THEN** ordinary cleanup obligations and strict actual-exit evidence remain required and the failure cannot be waived as startup cleanup

### Requirement: Boot identity uses the retained output of the declared build

The runner SHALL retain an exclusive executable snapshot of the successful declared BUILD output inside the current private append-only evidence root before TEST can refresh Cargo output. It SHALL record both paths and SHA256 values and refuse a copied-byte mismatch. Every main launch SHALL retain the same expected-path and built-byte checks against that snapshot, with compared values recorded even on refusal. It SHALL NOT adopt a replacement binary's later hash as new authority or silently disable identity checking.

#### Scenario: Cargo refreshes a target during test preparation

- **WHEN** TEST prepares artifacts after the declared BUILD has completed
- **THEN** the launched main candidate remains the exclusive snapshot of those BUILD bytes, with a matching recorded snapshot SHA and unchanged boot identity guard

#### Scenario: Copy or launch identity differs

- **WHEN** copied executable bytes or actual launch path/hash differ from their declared built identity
- **THEN** the runner refuses successful setup, retains the actual comparison, and does not execute or certify the differing main


### Requirement: Repair004 preserves exact failed-gate oracles
The verification fixture SHALL use the exact-profile driver constructor for its declared COUNT wire model, preserve final-byte SHA equality, and retry only explicitly opted-in bounded health polls for an owned main restart. KB collection creation SHALL use the actual mounted collection URL. CAS evidence SHALL require both observed winner directions and exact readbacks rather than infer commit order from launch order. Unchanged-target unrelated transaction conflicts SHALL remain errors; a later exact-key predicate mismatch may establish a CAS miss but SHALL NOT be called causal proof of the original conflict.

#### Scenario: Repeat winner does not prove both directions
- **WHEN** both controlled concurrent attempts yield the same observed winner direction
- **THEN** required two-direction coverage remains BLOCKED without an automatic retry or relaxed oracle

#### Scenario: Another-row transaction conflict leaves target unchanged
- **WHEN** the conditional transition receives a typed transaction conflict and the exact target predicate still matches or cannot be read conclusively
- **THEN** the original error is retained rather than reported as a CAS miss


### Requirement: Packet006 retains source-defined error and persistence proof
The gate SHALL separate agent and model selectors, parse terminal SSE objects without depending on whitespace, derive CAS control names from actual successful collaborator claims, and require the history presentation-owner namespace for exact snapshot cleanup. The non-team early budget-binding projection SHALL preserve its public budget category. The ingestion adapter SHALL preserve the underlying checked transaction cause before cancellation wrappers; only typed rolled-back transaction conflicts may repeat atomic acceptance, for at most3total attempts separated by25ms, with unchanged job/document/source identity and KB fence. Unknown commits and transport/other failures SHALL NOT retry.

#### Scenario: Underlying cause follows cancellation wrappers
- **WHEN** checked statement results contain cancellation wrappers before the real failure
- **THEN** the real typed failure is retained, with existing domain refusals unchanged

#### Scenario: CAS collaborator proves both required controls
- **WHEN** actual successful report claims confirm both winner coverage and unrelated conflict retention
- **THEN** both required control names are observed from that report without rerunning the collaborator or deleting an oracle
