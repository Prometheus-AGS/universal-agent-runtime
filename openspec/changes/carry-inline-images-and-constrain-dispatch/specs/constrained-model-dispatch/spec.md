## Purpose
Enforce hard payload and captured destination constraints on actual model sends using the existing routing and budget facilities. Advisory metadata alone does not certify execution capability.

## ADDED Requirements

### Requirement: Shared candidate admission
Advisory routing and actual dispatch SHALL use one explained capability/availability contract. Explicit registered disablement SHALL veto catalog supplementation. Preferences SHALL not override hard request requirements, cooldown or captured destination scope; missing required facts SHALL not be assumed certified.

#### Scenario: Preferred model is ineligible
- **WHEN** the preferred destination is disabled, cooling down or fails an actual tools/vision/structured/context requirement
- **THEN** it is not sent solely due to preference or catalog re-entry

### Requirement: Final-send scope enforcement
Every public primary, retry, fallback and graph send SHALL derive requirements from its final payload and stay within immutable actual captured model identities. Exact targets SHALL not substitute. Admission rejection SHALL not manufacture fallback authority or be treated as a provider outage.

#### Scenario: No eligible destination
- **WHEN** all captured candidates violate hard requirements or authority
- **THEN** execution explicitly refuses with no provider call

#### Scenario: Eligible fallback
- **WHEN** a permitted pre-output primary failure permits fallback
- **THEN** only a capable captured fallback is sent after fresh admission and its own destination preparation
- **AND** excluded or incapable fallback destinations receive zero calls

### Requirement: Preserve final-destination budget honesty
Dispatch SHALL preserve existing destination-bound endpoint/model/settings, output/reasoning reserves and protected groups. Unproven real-image accounting SHALL not claim guaranteed fit. Strict fit or finite accounting requiring an unproven image count SHALL refuse before dispatch; estimated lanes SHALL state their limit.

#### Scenario: Image count contract is unproven
- **WHEN** a real image request requires guaranteed fit without validated image-aware counting
- **THEN** it refuses before provider dispatch instead of substituting a flat allowance or JSON count as real image accounting

#### Scenario: Synthetic exact protected overflow
- **WHEN** an explicitly synthetic exact wire-count contract cannot fit the protected image/tool group and output/reasoning reserves
- **THEN** it refuses before send and the receipt credits only that synthetic contract

### Requirement: Actual execution evidence
Acceptance SHALL exercise actual mounted public runs and actual provider-adapter destinations with at least two deterministic registrations and controlled fallback. Route-query success, catalog declarations or compilation alone SHALL not establish dispatch behavior or provider quality.

#### Scenario: Capability fixture selects real send
- **WHEN** mounted public cases require tools, vision, structured output or bounded context
- **THEN** captured sends and zero-send controls prove only eligible destinations were used
- **AND** receipts retain exact source/config/artifact scope and distinguish real-model qualification
