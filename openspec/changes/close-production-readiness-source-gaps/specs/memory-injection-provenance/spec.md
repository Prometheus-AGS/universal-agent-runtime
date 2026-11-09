## ADDED Requirements

### Requirement: Report automatic memory selection and inclusion honestly
Automatic injection SHALL preserve owner/TTL/optout/relevance controls, emit safe typed no-match/unavailable/refused/prepared outcomes, and never assert successful inclusion without final provider-input evidence. Tool recall SHALL remain independent.

#### Scenario: A fresh fact is eligible or excluded before a new conversation
- **WHEN** A fresh fact is eligible or excluded before a new conversation
- **THEN** The actual final request and provenance agree; disabled/foreign/expired/deleted/below-threshold facts are absent and diagnostics contain no fact body
- **AND** an included fact from the current three-lane hybrid path records `owner_scoped_three_lane_hybrid_cosine_eligible` with its exact owner-scoped memory identity
- **AND** an empty retained owner-scoped candidate set records `memory_no_retained_scope_match`

#### Scenario: Submission digest preserves the exact request bytes
- **WHEN** a prepared memory selection survives the final provider request projection
- **THEN** its submission receipt uses SHA-256 over the existing serialized messages and anthropic_system payload, encoded as two lowercase hexadecimal characters per digest byte, and regression assertions compare the complete digest without dropping request, receipt, provenance or privacy conditions
