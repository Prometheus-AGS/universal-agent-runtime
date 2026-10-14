# dot-durable-budgets Specification

## Purpose
Define budget admission uses shared durable reservations for reliable always-on assistant operation and honest support claims.

## Requirements

### Requirement: Budget admission uses shared durable reservations

Aggregate budget admission SHALL atomically reserve durable shared capacity and restore spend and unresolved reservations across restart. Unknown liability SHALL remain reserved until reconciled.

#### Scenario: Concurrent admission

- **WHEN** two runtimes compete for insufficient remaining aggregate budget
- **THEN** atomic reservation admits only work within the configured bound

#### Scenario: Restart with unknown spend

- **WHEN** a runtime restarts while prior work has unresolved spend
- **THEN** enforcement restores the reservation and does not reset the available budget or release unknown liability
