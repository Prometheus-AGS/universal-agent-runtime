# spec-conformance-measurement

## ADDED Requirements

### Requirement: The capability matrix asserts contracts the runtime implements

Each capability case MUST assert the contract the runtime actually implements,
not the contract the test author expected. A case that fails because its own
assertion is wrong manufactures a false defect, and a measurement instrument
whose error rate exceeds the subject's cannot be trusted to measure it.

Where a capability is deliberately absent or deliberately retired, the case MUST
assert that absence explicitly — status and the discriminating body field — and
its name MUST carry a prefix that matches what it asserts.

#### Scenario: A guarded endpoint

- **GIVEN** a capability whose endpoint requires authentication
- **WHEN** the case exercises it
- **THEN** the case either presents a valid token and asserts success, or
  asserts the guard response as the contract
- **AND** the assertion message names which contract is being asserted

#### Scenario: A retired route

- **GIVEN** a route the runtime has deliberately retired
- **WHEN** the case exercises it
- **THEN** the case asserts the retirement status and its discriminating body
  field, not a success status
- **AND** the case name carries the `absent_` prefix

### Requirement: The capability matrix runs on a blocking CI gate

The matrix MUST execute in CI on a job that cannot pass while a case fails.
Compiling the test tier is not executing it: a compile gate proves the
instrument builds, and says nothing about whether the runtime works.

The gate's failure path MUST be demonstrated, not assumed. A gate that has only
ever been observed passing is indistinguishable from a gate that cannot fail.

#### Scenario: A failing case blocks the pipeline

- **GIVEN** the matrix job is wired with `continue-on-error` absent or false
- **WHEN** any capability case fails
- **THEN** the job fails
- **AND** the job log names the specific case that failed

#### Scenario: The gate is proven before it is trusted

- **GIVEN** a newly added matrix job
- **WHEN** a deliberate break is introduced in a named case and pushed
- **THEN** the job goes red and the log names that case
- **AND** reverting the break returns the job to green
- **AND** both run URLs are recorded in the change's verification record
