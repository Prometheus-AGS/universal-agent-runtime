## ADDED Requirements

### Requirement: Evidence classes carry limits

Public testing documentation SHALL state what each evidence class proves and
what it does not prove. Results SHALL identify their source SHA and applicable
profile when making a behavior claim, and MUST NOT transfer to another profile
without separate evidence.

#### Scenario: Reader evaluates a passing test

- **WHEN** a test result is presented as evidence
- **THEN** the documentation identifies the exercised boundary, source, profile, and explicit non-claims

### Requirement: Inference evidence crosses a genuine model boundary

Only a request that traverses a supported packaged UAR boundary, reaches a real
loaded model through the configured provider path, performs inference, and
returns the result through UAR MAY support an inference integration claim.
Synthetic, mocked, stubbed, recorded, replayed, or hard-coded responses MUST be
described as non-certifying diagnostics.

#### Scenario: Recorded provider test passes

- **WHEN** a recorded or synthetic provider returns a successful response
- **THEN** the result may support protocol or orchestration diagnostics but does not certify model inference, soak, resilience, release, or production readiness

### Requirement: Fail-closed claims include observed negative controls

Every fail-closed requirement SHALL pair its passing assertion with an observed
failing negative control, a bounded mutation, exact restoration, and retained
command/output evidence.

#### Scenario: Guard always passes

- **WHEN** deliberate inversion does not make the assertion fail
- **THEN** the fail-closed claim remains unverified even if its positive path passes
