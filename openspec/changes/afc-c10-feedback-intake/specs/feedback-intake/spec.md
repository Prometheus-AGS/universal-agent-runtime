## Purpose

Preserve one scoped feedback identity across model classification, duplicate detection, review and governed external effects.

## ADDED Requirements

### Requirement: Idempotent feedback observation and duplicate gate
UAR SHALL persist source identity and workflow linkage before external effects. It SHALL classify and draft through the pinned C10.1 run, and SHALL mark same-scope classified duplicates before issue authorization.

#### Scenario: Direct and channel replay
- **WHEN** the same source event is observed again or equivalent feedback arrives from direct and channel ingress
- **THEN** one canonical intake is eligible for an issue and repeated commands return the prior record.

### Requirement: Review and implementation are separate
UAR SHALL link only same-scope C09 product, design and reviewer artifacts. An issue receipt SHALL NOT authorize implementation; a separate explicit human decision SHALL name the exact artifacts and be durable.

#### Scenario: Issue created
- **WHEN** a governed issue is confirmed
- **THEN** implementation remains denied until an authenticated operator records the separate admission.
