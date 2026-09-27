# governed-action-boundary Specification

## Purpose
TBD - created by archiving change afc-c02-governed-action-boundary. Update Purpose after archive.

## Requirements

### Requirement: Every effect uses one exact authority envelope

UAR SHALL resolve a descriptor, validate arguments, and bind resource, payload, expected policy, grant, full UAR lease facts, and full UAR budget reservation facts before requesting approval or dispatching an effect. A downstream authority SHALL resolve its own current policy rather than treating UAR's observed policy revision as authoritative.

The lease and budget reservation SHALL have finite expirations. A configured bounded run timeout governs when present; otherwise UAR SHALL use the documented 300-second tool-approval TTL.

#### Scenario: Payload changes after approval

- **WHEN** the canonical validated payload differs from the payload bound to an approval
- **THEN** claim revalidation fails and the effect is not dispatched

#### Scenario: Authority changes during a wait

- **WHEN** policy, grant, lease, budget, runtime epoch, host epoch, or approval authority changes while an invocation waits
- **THEN** UAR re-evaluates the current authority immediately before claim and denies stale authority

### Requirement: UAR owns the effect lifecycle

UAR SHALL persist claim intent before dispatch and SHALL record terminal success, failure, interruption, or outcome-unknown without delegating execution ownership to the approval host.

#### Scenario: Terminal persistence is uncertain

- **WHEN** dispatch may have occurred but the terminal result cannot be durably recorded
- **THEN** UAR retains outcome-unknown and does not report rollback or retry the invocation identity

### Requirement: All callable entry points share admission

Direct REST, SDK, embedded, model tool-loop, graph delegation, and authenticated actor collaboration paths SHALL use descriptor resolution, argument validation, prepare, admission, claim, dispatch, and finish before producing an effect.

#### Scenario: Authenticated direct caller supplies another agent identity

- **WHEN** a direct REST caller supplies a caller-controlled agent identity header
- **THEN** UAR derives the effect principal from the middleware-verified subject and does not evaluate or dispatch the effect as the supplied identity

#### Scenario: Actor collaboration requires approval

- **WHEN** actor collaboration prepares a required `spawn_agent` effect or the host returns `ask`
- **THEN** UAR uses the root run's approval gate and dispatches no child unless that gate returns an admitted decision

#### Scenario: Direct REST execution

- **WHEN** an authenticated caller invokes `/api/tools/{name}/execute`
- **THEN** the call uses the same exact-invocation admission and lifecycle as a model tool call

### Requirement: Governed policy loading fails closed

A governed server profile SHALL reject missing, unreadable, empty, or invalid policy input and SHALL NOT replace it with permit-all authority.

#### Scenario: Policy directory is empty

- **WHEN** governed startup finds no readable Cedar policy
- **THEN** startup fails with a visible policy error before callable ingress becomes active

### Requirement: Local bypass is explicit and constrained

Governance bypass SHALL be available only for a runtime-proven loopback standalone root with no delegated actor or paired host, and SHALL remain visible in runtime status and admission disposition.

#### Scenario: Delegated actor attempts standalone bypass

- **WHEN** a delegated or host-paired execution lacks active governance authority
- **THEN** UAR denies it rather than using standalone admission
