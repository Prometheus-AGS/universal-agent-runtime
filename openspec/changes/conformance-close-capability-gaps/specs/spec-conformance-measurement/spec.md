# spec-conformance-measurement

## ADDED Requirements

### Requirement: Every capability carries a result or a published exclusion

Every capability declared in `docs/SPECIFICATION.md` MUST have either a case
meeting a stated minimum evidence level, or a published exclusion naming the
reason it cannot be exercised. A capability that is silently absent from the
matrix is indistinguishable from one that passes, which is the failure mode this
measurement exists to prevent.

Coverage MUST NOT be reported as conformance. The count of capabilities having a
case says nothing about what those cases establish.

#### Scenario: A capability that cannot be exercised

- **GIVEN** a capability requiring conditions the harness cannot create
- **WHEN** the matrix is assembled
- **THEN** an `excluded_` case exists naming the blocking condition
- **AND** the exclusion appears in the published result beside the measured
  capabilities, not in a footnote

#### Scenario: A security capability falls short of its target

- **GIVEN** a capability whose target evidence level is L3-plus-negative
- **WHEN** only a shape assertion is achievable
- **THEN** it is published as an exclusion with that reason
- **AND** it is NOT recorded as a pass at a lower level

### Requirement: Evidence labels reflect what was exercised

Each case name MUST carry a prefix from a closed, documented set, and the prefix
MUST reflect what the case actually establishes. Against a stub whose fixtures
the test author wrote, a case whose correctness depends on stub output is wired,
not exercised, and MUST NOT claim L3.

#### Scenario: A case depends on stub output

- **GIVEN** a case whose assertion passes only because the stub returned the
  fixture the test author wrote
- **WHEN** its evidence level is assigned
- **THEN** it is labelled `l2_`
- **AND** relabelling from a higher level records the before and after

#### Scenario: A case proves only that a route resolves

- **GIVEN** a case asserting that a request did not hit the `/api/{*path}`
  catch-all
- **WHEN** its evidence level is assigned
- **THEN** it is labelled `l1_` and does not satisfy any target above L1

### Requirement: A discriminator proves the real handler answered

The runtime mounts a catch-all at `/api/{*path}` returning
`code: "api_route_not_found"`. A case asserting only "not 404" cannot
distinguish a mounted route from the catch-all, so every case MUST assert a
discriminator — a status, header, or body field the real handler produces.

#### Scenario: A route is absent but the catch-all answers

- **GIVEN** a capability whose route is not mounted
- **WHEN** the case issues its request
- **THEN** the discriminator fails and the case reports absence
- **AND** the case does not pass on the strength of a non-404 status
