## ADDED Requirements

### Requirement: Architecture decision history routes

The portal SHALL publish an architecture-history overview, complete retained ADR
index, dated timeline, correction ledger, and process-provenance guide. Current
product guides SHALL remain authoritative for present behavior.

#### Scenario: Reader enters the history section

- **WHEN** a reader opens the History category
- **THEN** the reader can navigate the five architecture-history guides and distinguish current, superseded, and historical records

#### Scenario: Accepted decision is mistaken for delivered behavior

- **WHEN** an ADR or plan is accepted but delivery is not established by current source or specification
- **THEN** the portal describes the record as intent or history rather than a delivered product claim
