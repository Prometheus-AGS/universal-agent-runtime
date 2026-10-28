## ADDED Requirements

### Requirement: Destination counting claims require the reviewed contract

Request preparation SHALL use the reviewed destination wire, counting and capacity contract for exact-fit claims. Approximate or unsupported counters and unverified gateway capacity SHALL NOT be reported as destination-exact evidence. The evidence SHALL distinguish the trusted Rust builder from standalone-main exact-fit support, which remains UNSUPPORTED in this scope.

#### Scenario: Trusted builder boundary evidence

- **WHEN** COUNT-01 exercises the reviewed trusted Rust builder with actual final-wire observations
- **THEN** its receipt identifies evidenceScope as trusted_rust_builder and standaloneMainExactFit as UNSUPPORTED rather than claiming a standalone-main runtime capability

#### Scenario: Counter or capacity lacks destination proof

- **WHEN** counting is approximate or unsupported, or the selected gateway capacity is unverified
- **THEN** the outcome does not receive a provider-exact or standalone-main exact-fit PASS

### Requirement: Final preparation accounts for tools and protected reserve

Preparation and trimming SHALL include tool payloads and output reserve in the reviewed capacity calculation and SHALL preserve protected content. Final dispatch SHALL honor that reviewed boundary before producing an upstream effect.

#### Scenario: At-limit request dispatches once

- **WHEN** the final trusted-builder request including tools, protected content and output reserve fits the reviewed capacity at its limit
- **THEN** the controlled destination observes exactly one dispatch with the protected content and reserve retained

#### Scenario: Over-limit request refuses before dispatch

- **WHEN** the final request exceeds the reviewed limit after the supported preparation and trim steps
- **THEN** admission refuses before upstream dispatch and the controlled effect count remains zero
