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

### Requirement: The capability matrix is a mandatory local gate

The matrix MUST execute locally with the pinned command before a conformance
change is considered complete or its commit is pushed. A non-zero result MUST
block completion. Compiling the test tier is not executing it: a compile check
proves the instrument builds, and says nothing about whether the runtime works.

GitHub Actions MUST NOT run the matrix or other unit, integration, conformance,
lint, format, or routine development checks. GitHub Actions are reserved for
deployment and deployment validation.

The gate's failure path MUST be demonstrated, not assumed. A gate that has only
ever been observed passing is indistinguishable from a gate that cannot fail.

#### Scenario: A failing case blocks local completion

- **GIVEN** the pinned matrix command is run locally
- **WHEN** any capability case fails
- **THEN** the command exits non-zero
- **AND** its output names the specific case that failed

#### Scenario: The gate is proven before it is trusted

- **GIVEN** the pinned matrix command
- **WHEN** a deliberate break is introduced in a named case and run locally
- **THEN** the command exits non-zero and its output names that case
- **AND** reverting the break returns the local command to green
- **AND** both command results are recorded in the change's verification record
