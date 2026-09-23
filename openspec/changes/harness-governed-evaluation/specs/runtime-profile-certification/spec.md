## Purpose

Make runtime compatibility claims depend on reproducible local evidence for affected supported profiles and explicitly tracked verification debt.

## ADDED Requirements

### Requirement: Certification names supported profiles and evidence
A phase claim/evidence matrix SHALL cover F1–F8 and default/minimal, server-full, desktop-full and embedded/mobile profiles at their declared support levels. Linux and macOS stable support and Windows experimental status SHALL be distinguished using current architecture policy. Every affected behavior SHALL have an applicable profile test, an evidenced unsupported designation or an environment-blocked result; missing evidence for a supported required path SHALL block completion. Existing streaming resume/dedup, graph events, telemetry/cost, cancellation and shutdown claims SHALL be included.

#### Scenario: Only server-full passes
- **WHEN** server-full passes while another affected supported profile lacks evidence
- **THEN** the phase remains incomplete and the missing profile is named; a server result cannot stand in for all profiles

#### Scenario: Experimental or inapplicable surface
- **WHEN** a profile lacks a surface by declared architecture or has experimental support
- **THEN** the matrix records the source for that status and the applicable contract rather than inventing a passing test or silently dropping the row

### Requirement: Verification debt is remeasured without weakening gates
The two historically reported eval failures SHALL be identified and reproduced or classified with current evidence. Frontend coverage SHALL be remeasured using the declared metric, numerator, denominator and exclusions against the existing 60 percent target; the historical 33.68 percent SHALL remain a reported value until reproduced. Required thresholds and exclusions SHALL NOT be weakened to pass. Missing historical reproducer information SHALL remain explicit verification debt.

#### Scenario: Current coverage differs from report
- **WHEN** current local coverage is measured
- **THEN** the report records exact command, revision, environment, metric, exclusions and result, distinguishes historical values and fails completion if the applicable 60 percent target is not met

#### Scenario: Historical test no longer fails
- **WHEN** a named prior failing case now passes on the declared environment
- **THEN** evidence links the original claim and current result without asserting an unobserved fix or dropping the other failure

### Requirement: Verification tiers and local milestone gate are explicit
Tier 0 SHALL run after each edit, Tier 1 at unit completion and Tier 2 at phase completion under repository rules. Tier 3 supported-profile certification SHALL occur only in a separately scoped local certification milestone registered during Plan and SHALL be a prerequisite for this phase's completion, not an implicit release or deployment. Tests SHALL respect one writer per build/target directory. Previously interrupted builds SHALL remain unknown until successfully rerun at the proper tier.

#### Scenario: Certification milestone unavailable
- **WHEN** required milestone hardware, credentials or supported environments are unavailable
- **THEN** those rows remain blocked and neither the milestone nor this phase is reported complete

#### Scenario: Spec-only stage
- **WHEN** only proposal, design and specification artifacts are edited
- **THEN** structural/spec validation is reported as artifact verification, no runtime certification is inferred and the pending prior readiness reflection remains pending

