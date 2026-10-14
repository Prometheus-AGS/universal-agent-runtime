# dot-spend-settlement Specification

## Purpose
Define usage settlement has stable durable identity for reliable always-on assistant operation and honest support claims.

## Requirements

### Requirement: Usage settlement has stable durable identity

Usage settlement SHALL be durably idempotent for each logical call and accounting scope. Execution outcome and unresolved accounting SHALL remain separately visible.

#### Scenario: Repeated settlement

- **WHEN** a terminal usage result is persisted twice or replayed after a crash
- **THEN** each applicable budget scope records the logical charge once

#### Scenario: Accounting outage

- **WHEN** execution completes but usage settlement cannot be durably accepted
- **THEN** execution truth is preserved and accounting remains visibly unresolved rather than reported settled
