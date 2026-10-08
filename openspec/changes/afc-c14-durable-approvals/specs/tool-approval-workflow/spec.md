# ADDED Requirements

## Requirement: Retained authoritative approval decisions
The runtime SHALL expose an issuer-scoped challenge independently of its authoritative decision and tool-effect lifecycle. One authenticated decision SHALL be persisted atomically, and other authorized clients SHALL read the same result after resolution and reopening. Tenant and admitted-workspace boundaries SHALL be preserved.

### Scenario: Two clients resolve one challenge
- WHEN two authorized clients observe one challenge and one resolves it
- THEN both read the same retained decision and a competing resolution cannot replace it.

### Scenario: Historical issuer
- WHEN a client reads a pending challenge from another issuer or without a live waiter
- THEN its observed state and actual issuer remain visible, resolution is disabled on this runtime, and no denial, interruption or effect replay is inferred.

### Scenario: Observed gate termination
- WHEN the issuing broker observes cancellation, deadline expiry or channel closure
- THEN it persists cancelled, expired or interrupted respectively unless a human decision already committed, without manufacturing a human denial.

## Requirement: Separate observation and executor controls
The client SHALL distinguish detach observation from stopping execution, and SHALL retain supported cancel and drain semantics.

### Scenario: Detach and stop
- WHEN one client detaches its output observation
- THEN the executor remains alive and reattachment resumes its cursor; explicit stop uses authoritative cancellation.
