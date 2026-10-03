## Purpose

Provide durable, owner-scoped feedback classification and internal drafts with pinned definitions and exact-artifact human decisions.

## ADDED Requirements

### Requirement: Closed pinned interpretation
The runtime SHALL activate only the declared version 1.0.0 classify/draft profile, pin its complete immutable definition and compiled plan, enforce activation limits and reject unsupported execution semantics.

#### Scenario: Unsupported workflow
- **WHEN** a definition requests loops, joins, retry, effects or an unknown mapping
- **THEN** activation is refused with an inspectable field diagnostic

#### Scenario: Definition update during wait
- **WHEN** a newer definition is installed while a run waits and the runtime restarts
- **THEN** the original plan, draft and wait remain unchanged

### Requirement: Single durable execution owner
The runtime SHALL use C09 admission and dispatch, persist the two canonical task identities before execution, and commit original attempt links atomically with admission. Only successful authoritative outcomes with exact validated artifacts and confirmed effects permit progression. Unknown accounting SHALL retain reservations without erasing known successful execution.

#### Scenario: Duplicate progression
- **WHEN** competing or repeated progression requests observe the same ready step
- **THEN** one original attempt is admitted and claimed, with no duplicate model execution

#### Scenario: Unknown execution or insufficient remaining budget
- **WHEN** execution is uncertain or held reservations prevent the next admission
- **THEN** the original identities and reason remain inspectable and no dependent turn is dispatched

### Requirement: Scoped exact decisions and controls
The runtime SHALL authorize all operations by authenticated owner/workspace and current binding/member authority. It SHALL bind command IDs to complete request digests and accept a decision only for the current revision, wait, artifact ID and digest. Acceptance SHALL finalize an internal artifact without an effect or model turn. Cancellation SHALL preserve unresolved original execution until terminal cleanup is known; recovery SHALL check that no producer is live before reconciliation.

#### Scenario: Repeated or stale decision
- **WHEN** a decision is repeated identically or with conflicting content
- **THEN** the identical request returns its receipt and the conflicting request is refused

#### Scenario: Other workspace or revoked authority
- **WHEN** a different workspace inspects/decides a run or revoked authority advances/accepts it
- **THEN** the operation is refused without exposing scoped artifacts

### Requirement: Explicit selected context
Each workflow attempt SHALL receive only its pinned step instructions, bounded workflow input and explicitly selected predecessor artifact. It SHALL have no executable tools or implicit team inbox/history context.

#### Scenario: Draft follows classification
- **WHEN** the classifier commits a valid artifact
- **THEN** draft receives that exact artifact and feedback, while unrelated team context is absent
