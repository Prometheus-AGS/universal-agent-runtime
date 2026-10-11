# Spec Delta

## Purpose

Ensure every approved effect belongs to one authenticated execution and invocation, and that HTTP MCP session reuse cannot cross principal boundaries.

## ADDED Requirements

### Requirement: Exact root approval identity
Migrated root approval clients SHALL submit the exact pending approval identity. Decisions MUST be bound to the authenticated owner, run and prepared invocation. Full-harness decisions MUST also match the expected task revision. Missing identity MUST NOT select the currently pending request implicitly.

#### Scenario: Missing or stale approval
- **WHEN** an approval omits identity, targets another owner/run/invocation, repeats a consumed decision, or uses a stale task revision
- **THEN** it is rejected without authorizing an effect or consuming the current legitimate pending decision.

#### Scenario: Invocation changes after display
- **WHEN** tool arguments, destination, policy revision or credential lease differ from the approved prepared invocation
- **THEN** the effect is not admitted under the old approval; any new decision requires a newly prepared identity.

#### Scenario: Cancellation races with approval
- **WHEN** a decision arrives after cancellation or terminal cleanup
- **THEN** it cannot revive execution or authorize another invocation, and an in-flight effect retains its actual or unknown outcome.

### Requirement: Approval compatibility is explicit
The migration SHALL inventory root approval callers and declare a cutover checkpoint. Remote and new harness clients MUST require exact identity. Any temporary trusted-local compatibility MUST be explicitly approved, narrowly identified and removable; it MUST NOT weaken host claim-before-effect.

#### Scenario: Legacy client after cutover
- **WHEN** an old client sends a run-only approval to a strict endpoint
- **THEN** it receives an actionable incompatibility result with no effect; the system does not silently downgrade authorization.

### Requirement: Single claim for a prepared effect
The effect boundary SHALL claim an admitted invocation once using its bound owner, run, payload, destination, policy and lease. Ambiguous completion MUST remain unknown and MUST NOT be automatically replayed.

#### Scenario: Concurrent duplicate claims
- **WHEN** two requests race to execute the same admitted invocation
- **THEN** no more than one reaches the effect and the other receives an explicit already-claimed or unavailable result.

#### Scenario: Lost effect response
- **WHEN** an effect may have completed but its response is lost
- **THEN** the invocation is not claimed again and its outcome remains unknown until authoritative reconciliation.

### Requirement: HTTP MCP transport sessions preserve principal ownership
An authenticated principal SHALL access only its own authorized HTTP MCP session operations, including continuation, observation, replay and deletion. Session identifiers MUST NOT confer authority. A rejected cross-principal operation MUST leave the original session usable.

#### Scenario: Authenticated session crossover matrix
- **WHEN** independently authenticated B uses A's valid session ID for POST continuation, GET observation or replay with A's event cursor, or DELETE
- **THEN** each operation returns 403 or non-disclosing 404, reveals no A response/event, produces no effect or A lifecycle mutation, and A's subsequent authorized control operation succeeds.

#### Scenario: Authentication is not bypassed on reuse
- **WHEN** a client reuses a valid session ID with missing or expired credentials
- **THEN** authentication rejects the request before session routing and no session data or effect is released.

### Requirement: Session remediation is evidence-gated
Session-binding remediation SHALL remain blocked until a real production-router two-principal scenario reproduces a violation. A passing matrix SHALL close the hypothesis only with the identified production enforcing layer, valid protocol requests, positive owner controls and negative effect/lifecycle assertions.

#### Scenario: Hypothesis is disproved
- **WHEN** the complete authenticated matrix passes on the existing transport
- **THEN** the receipt identifies the production module/function/code branch that compared authenticated owner with stored session owner (or equivalent tenant-bound session namespace), its 403/non-disclosing404 result and the observed valid A/B controls; JWT validity checks alone are insufficient. No speculative binding rewrite is performed; exact approval migration remains independently required.

#### Scenario: Hypothesis is reproduced
- **WHEN** the complete authenticated matrix demonstrates crossover
- **THEN** the failure is recorded before binding remediation and the same matrix must pass after the complete correction; generic 500 responses or invalid requests cannot count as a passing denial.
