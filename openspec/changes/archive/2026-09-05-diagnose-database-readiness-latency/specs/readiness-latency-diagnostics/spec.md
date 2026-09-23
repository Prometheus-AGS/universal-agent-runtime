## Purpose

Define a bounded operational diagnostic workflow that distinguishes recorded
readiness symptoms from evidence-backed causes without changing running services.

## ADDED Requirements

### Requirement: Diagnostic evidence identifies its actual target

The workflow SHALL record source revision, installed executable digest, process
identity/start time, listener identity, nonsecret effective persistence target
and request-timeout provenance before active observations. It SHALL distinguish
historical receipts from current observations and SHALL skip active probes when
identity or authentication cannot be established safely.

#### Scenario: Target identity cannot be confirmed
- **WHEN** effective configuration or installed identity remains unresolved
- **THEN** the report names the missing evidence and records active probes as skipped
- **AND** it does not substitute defaults or old process IDs as current evidence

### Requirement: Active observations are bounded and non-mutating

The workflow SHALL require explicit Execute authorization after planning and
SHALL use at most one active request, two rounds and 360 seconds of active
observation. It SHALL NOT change application records, code, dependencies,
configuration or service lifecycle, or send inference requests. It SHALL NOT
run product test suites or automatically retry a failed observation.

The total request ceiling SHALL be two database-health requests, four UAR
liveness requests, two UAR readiness requests and two authenticated database
sessions with at most five read queries each. Per-operation limits SHALL be
5 seconds for health, 35 seconds for readiness, 120 seconds per database session,
15 seconds per setup stage, and 10 seconds per query RPC. Active collection SHALL
end on the first timeout, error, identity drift, oversized response, observed
host degradation or operator interruption; remaining probes SHALL be marked skipped.

#### Scenario: Readiness times out
- **WHEN** the existing readiness operation exceeds its client deadline or returns a timeout
- **THEN** the workflow stops active collection and records the failure and skipped comparisons
- **AND** it does not treat closing the connection as proof of server-side cancellation

### Requirement: Diagnostic outputs protect credentials and business content

The workflow SHALL limit buffered responses to 256KiB each, log-tail reads to
256KiB per relevant file, and sanitized evidence files to 1MiB each. It SHALL
retain only allowlisted identity, timings, status, byte counts, scalar booleans,
row counts and categorized log evidence. Credentials, tokens, environment dumps,
full process arguments, record IDs, skill contents and embeddings SHALL NOT
appear in retained output. Diagnostic table reads SHALL project only IDs with
at most 32 returned rows and a five-second statement timeout; IDs SHALL be
discarded before output. New full-table, COUNT or EXPLAIN diagnostic scans are
prohibited; the existing readiness endpoint's full-table operation remains a
named risk subject to the readiness request ceiling.

#### Scenario: Response exceeds its permitted size
- **WHEN** a response exceeds 256KiB or has an unexpected shape
- **THEN** active collection ends with sanitized size/format failure evidence
- **AND** no raw response, token or business payload is retained

### Requirement: Attribution distinguishes measurement scopes and uncertainty

The report SHALL distinguish socket/authentication/namespace setup time, client
round trips and server-reported statement time. It SHALL assess full-row work,
setup overhead, shared host/database contention, application waiting and startup
effects using supporting and disconfirming evidence. It SHALL NOT infer total
table size from a limited result, causation from resource correlation, or network
time by subtracting measurements with different scopes.

#### Scenario: Small database reads are fast but readiness is slow
- **WHEN** bounded direct reads succeed faster than readiness
- **THEN** the report retains unresolved full-table transfer, application-client,
  scheduling and registry explanations unless separate evidence distinguishes them
- **AND** it does not label full-row decoding as the root cause by elimination

### Requirement: Acceptance is independent and does not imply recovery

The diagnostic deliverable SHALL include execution evidence, hypothesis
dispositions, a minimal justified recommendation or next required authority,
verification and an operator handoff. A fresh-context artifact-only reviewer
SHALL evaluate the diagnosis and evidence before acceptance. Unresolved critical
review findings SHALL prevent unqualified acceptance. An inconclusive report
SHALL name each unavailable comparison and leave unresolved causal goals unmet.
Reflection SHALL follow Execute acceptance. Sampled success SHALL NOT be reported
as sustained readiness, repaired service or release certification.

#### Scenario: Safe observations cannot isolate the cause
- **WHEN** bounded evidence leaves multiple explanations and more evidence requires new authority
- **THEN** the report is explicitly inconclusive and names the smallest next diagnostic action
- **AND** neither a root cause nor recovered readiness is claimed
