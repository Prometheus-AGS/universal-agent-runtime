# Spec Delta

## ADDED Requirements

### Requirement: Run-binding diagnostics are typed and secret-safe

**Stable task:** R1. The trusted host SHALL classify a refused run-model binding with an allowlisted typed stage and category plus a diagnostic reference. It SHALL preserve the existing `orchestrator_start_failed` public code and AG-UI `RUN_ERROR` code/message shape, and SHALL NOT project an arbitrary source error, endpoint, credential, prompt, provider response, or effective model configuration.

#### Scenario: Bounded run lacks durable-accounting eligibility

- **WHEN** an explicitly bounded run reaches durable accounting preparation and the selected backend is unsupported or lacks a verified accounting principal
- **THEN** the trusted host rejects the run before provider or tool effects with `orchestrator_start_failed`, an allowlisted stage/category and diagnostic reference in its existing message projection
- **AND** the AG-UI projection remains a `RUN_ERROR` carrying only the existing code/message fields
- **AND** no untrusted kernel receives a provider rebinding, accounting mutation, or diagnostic-receipt write capability

#### Scenario: Captured binding cannot construct a client

- **WHEN** the trusted host cannot construct the captured primary client after policy intersection and budget admission
- **THEN** it reports an allowlisted client-construction category and diagnostic reference through the existing public failure projection
- **AND** it does not format or expose the constructor error chain

#### Scenario: Diagnostic origin is safe to log

- **WHEN** a run-binding refusal is recorded for operator diagnosis
- **THEN** structured logging contains only the reference, typed stage/category, `accounting_required`, `durable_backend_supported`, and `verified_principal_present` where applicable
- **AND** it does not contain credential values, endpoint URLs, provider bodies, raw configuration, or formatted source errors

#### Scenario: Existing team binding outcome is retained

- **WHEN** a team-profile binding returns an existing explicit team outcome
- **THEN** the existing team code and meaning remain unchanged
- **AND** the ordinary diagnostic category does not replace that team outcome

#### Scenario: Persisted bounded agents lack accounting storage at startup

- **WHEN** startup loads persisted token/cost-limited agents and its store cannot support durable accounting
- **THEN** it logs a warning with an allowlisted category, captured eligibility and bounded-agent count before accepting requests
- **AND** it neither weakens those budgets nor treats that startup scan as authorization for later runs

#### Scenario: Conservative call reservation exceeds the configured envelope

- **WHEN** eligible durable accounting refuses a call whose catalog context-plus-output reservation exceeds an explicit 16,000-token envelope
- **THEN** no provider contact occurs and the existing public budget-admission failure remains intact
- **AND** an ERROR log records the safe model-call-admission stage, category, typed accounting cause and diagnostic reference, reservation source/bound and configured token limits
- **AND** arbitrary error chains, credentials, endpoints and request content remain absent
