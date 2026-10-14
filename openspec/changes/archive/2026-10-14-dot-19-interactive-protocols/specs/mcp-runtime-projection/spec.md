# Spec Delta

## ADDED Requirements

### Requirement: MCP client elicitation uses a governed run interaction

Every supported UAR MCP client transport SHALL advertise the selected pinned elicitation capability only when its run-scoped host interaction channel can handle it. An incoming `elicitation/create` SHALL have a durable owner/run-bound accept, decline or cancel disposition. Elicitation acceptance SHALL NOT by itself grant permission to call or retry a tool; ordinary descriptor, policy and approval admission remains authoritative.

#### Scenario: Accept, decline and cancel

- **WHEN** an MCP server requests elicitation during an authorized run
- **THEN** the request reaches that run's governed interaction channel and returns the selected protocol's accept, decline or cancel result from a committed disposition

#### Scenario: Accepted elicitation precedes a Required tool

- **WHEN** the user accepts elicitation and the MCP server subsequently requests a Required tool effect
- **THEN** the tool still waits for its own host approval and cannot execute solely because elicitation was accepted

#### Scenario: Disconnected or restarted client

- **WHEN** the MCP transport disconnects or the runtime restarts while elicitation is pending
- **THEN** the interaction remains durably pending or reaches an explicit terminal outcome, without silent acceptance or automatic replay of an uncertain effect

#### Scenario: Run partition and frozen local children

- **WHEN** two roots use the same owner, server configuration and environment
- **THEN** an enabled elicitation handler is partitioned by its host channel identity, while a local child inherits only the original frozen root channel and narrowed tool bindings
- **AND** completing the root revokes and joins that channel's connections without closing another root's channel

#### Scenario: Input waits preserve absolute authority lifetimes

- **WHEN** a human input waiter is pending
- **THEN** the ordinary MCP execution timer pauses within the ten-minute interaction lifetime
- **AND** root cancellation, request abandonment and credential revocation or expiry continue to prevent response delivery without extending any tool or delegation authority

#### Scenario: Newer SDK multi-round behavior cannot replay tools

- **WHEN** a peer returns an unsupported multi-round input-required tool result
- **THEN** UAR does not drive SDK-managed follow-up calls and reports the original invocation through its existing non-success outcome contract

#### Scenario: Cold history remains inspectable

- **WHEN** an authenticated owner reads a pending interaction after the original host process died
- **THEN** the exact durable request remains visible with no resolvable live waiter, and a response cannot be accepted into a replacement root or connection
- **AND** expiry is persisted on subsequent history reads without creating an approval or execution record
