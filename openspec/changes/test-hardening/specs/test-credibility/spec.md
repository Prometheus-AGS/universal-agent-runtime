## ADDED Requirements

### Requirement: Proven suites gate CI and e2e specs assert real outcomes

Load-bearing test suites that are reliably green SHALL run as blocking CI gates,
and end-to-end specs SHALL assert real outcomes rather than mere element
visibility or treat failures as passes.

#### Scenario: A regression in a proven suite fails CI

- **Given** the BDD chat suite and the recorded-backend live integration tier
  are green
- **When** a change regresses either suite
- **Then** the corresponding workflow MUST fail (not `continue-on-error`)

#### Scenario: An e2e spec rejects a failed response

- **Given** the RAG e2e smoke sends a query
- **When** the assistant returns an empty response or an error state
- **Then** the spec MUST fail, not pass
