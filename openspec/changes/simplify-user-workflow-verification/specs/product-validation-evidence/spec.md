## ADDED Requirements

### Requirement: User workflows have independent admission
The verification runner SHALL admit a selected workflow/platform/backend cell using only its declared prerequisites. An unavailable fixture or platform SHALL block only dependent cells. The full coverage registry SHALL retain every advertised capability and map every prior acceptance/supporting obligation, with explicit reasons for deduplication or non-applicability.

#### Scenario: Knowledge or mobile preparation is unavailable
- **WHEN** the configured-prompt cell has its real provider, supported store and actual platform artifact but KB or mobile preparation is unavailable
- **THEN** the configured-prompt cell can run under explicit execution authority and the unavailable dependent cells remain BLOCKED
- **AND** their absence is not counted as PASS or removed from the denominator

### Requirement: Useful outcomes and failure collection remain truthful
Positive functional claims SHALL use genuine configured inference and task-specific observable predicates. Results SHALL preserve initial failures, cleanup evidence and exact source/artifact cohorts. At most one bounded authorized correction and same-item rerun SHALL occur; independent work SHALL continue when its prerequisites and resource safety permit.

#### Scenario: A cell fails and its correction does not resolve it
- **WHEN** the cell fails and its single authorized correction/rerun still fails
- **THEN** both attempts remain recorded, an unresolved failure is logged, and independent cells continue
- **AND** a changed-source PASS never certifies a previous or unexercised source cohort

### Requirement: Simplification does not create authority or erase history
The child SHALL preserve parent evidence, cumulative consumption, UNKNOWNs and separate provider ceilings. The replacement policy and exact commands SHALL require explicit operator adoption/approval after implementation; planning or child activation SHALL not open execution.

#### Scenario: A child phase starts while parent execution is stopped
- **WHEN** the verification redesign child is activated
- **THEN** design work proceeds with execution CLOSED and previous consumption/failures retained
- **AND** no signature, test, build, service or provider request is implied

### Requirement: Complete platform sweeps in operator order
Verification SHALL finish the complete applicable macOS sweep before Linux, then finish Linux before iOS. Within-platform independent cases SHALL continue after logged failures when their prerequisites and resource safety permit. Every runnable cell SHALL be attempted and every blocked dependent explicitly recorded before advancement. Sweep completion SHALL not be reported as platform certification when required failures remain.

#### Scenario: macOS reveals a shared product defect
- **WHEN** the macOS sweep exposes a shared defect and an authorized bounded correction is made
- **THEN** its original failure and rerun remain recorded and later Linux/iOS artifacts use the corrected source
- **AND** the remaining applicable macOS checks finish before Linux begins

#### Scenario: A later platform fix changes previously verified behavior
- **WHEN** a Linux or iOS correction changes shared behavior previously exercised on an earlier platform
- **THEN** affected earlier-platform claims require targeted regression evidence or remain explicitly unverified for the changed source
- **AND** unchanged platform-independent evidence is not needlessly rerun and platform runtime results never transfer

### Requirement: Functional scenarios do not perform authentication
Manual functional verification SHALL use an externally provisioned identity endpoint and SHALL NOT require, issue, renew or send authentication credentials from the test scripts. Nonsecret model, endpoint and workspace selection remain supported. The external runtime host preserves owner/tenant context for persistent records; production identity boundaries are not disabled.

#### Scenario: Team behavior without test credentials
- **WHEN** the existing math/writing BDD runs with only endpoint, model, workspace and report settings
- **THEN** external infrastructure supplies owner context without test-managed authentication
- **AND** the original concurrency, arithmetic, editorial result, accounting and merged-article assertions remain unchanged
