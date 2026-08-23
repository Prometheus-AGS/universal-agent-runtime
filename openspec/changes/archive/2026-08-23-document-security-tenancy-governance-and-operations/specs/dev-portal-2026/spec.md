## ADDED Requirements

### Requirement: Security and operations guide set

The developer portal SHALL publish source-grounded guides for authentication,
credentials, tenant isolation, governance, approvals, the runtime console, run
operations, observability, realtime state, cost controls, and recovery and
shutdown. The guides SHALL link the shipped operator surfaces to their API or
process boundaries and SHALL identify unsupported or process-local behavior.

#### Scenario: Operator follows the operating contract
- **WHEN** an operator follows the security and operations guides
- **THEN** the portal identifies how to configure and observe each delivered control, its fail-closed or permissive behavior, and the profile and durability limits that apply

#### Scenario: Required operating guide is absent
- **WHEN** a required guide or compatibility route is removed
- **THEN** local documentation validation exits non-zero and identifies the missing document or route

### Requirement: Authentication and credential boundaries

The authentication guide SHALL distinguish required and optional JWT behavior,
HS256 and RS256/JWKS verification, issuer/audience/not-before validation,
RustCrypto process-provider ownership, API-key creation and exchange, and
unauthenticated probe exceptions. The credential guide SHALL distinguish API
keys from provider credentials, document write-only plaintext and masked reads,
state the credential resolution order, and explain the encryption-key and
single-tenant fallback boundaries without publishing usable secrets.

#### Scenario: Authentication verification fails
- **WHEN** a required token is absent, has an invalid signature or registered claim, uses an unsupported JWKS algorithm, has an unknown key after refresh, or cannot be verified because the JWKS endpoint is unavailable with no cached key
- **THEN** the guide states the request is rejected and does not describe an unverified claim as an identity or tenant boundary

#### Scenario: Provider credential service is unavailable
- **WHEN** the credential encryption key is absent or invalid
- **THEN** the guide distinguishes disabled per-user credential storage from the operator environment/configuration fallback and does not claim encrypted multi-tenant storage is active

### Requirement: Tenant, governance, and approval limits

The portal SHALL document tenant identity as a value created only from a
verified credential and SHALL enumerate the current subsystems that consume it
without claiming universal tenant partitioning. It SHALL distinguish Cedar
allow/deny decisions from risk-based human approval, document profile-specific
governance behavior and policy-load fallback, and state that denial, rejection,
channel closure, cancellation, and approval timeout do not authorize tool
execution.

#### Scenario: Tenant claim is unverified
- **WHEN** a request supplies a tenant-like string outside successful credential verification
- **THEN** the guide states that UAR does not construct a trusted tenant identity from that value

#### Scenario: Governance is unavailable or permissive
- **WHEN** a profile omits Cedar or the server falls back to a permissive policy set after a policy-load error
- **THEN** the guide states that Cedar denial is not provided by that condition and does not present the runtime as fail closed by default

#### Scenario: Approval is not completed
- **WHEN** a human rejects a tool call, its approval channel closes, its run is cancelled, or five minutes elapse without approval
- **THEN** the guide states that the tool call is rejected and that a Cedar denial cannot be overridden by approval

### Requirement: Observability, realtime, and cost evidence limits

The operations guides SHALL document health/readiness probes, Prometheus
metrics, structured logs, optional OTLP export, runtime-console entity state,
run inspection and cancellation, multiplexed realtime updates, reconnect
behavior, usage-derived cost estimates, and budget alerts. They MUST distinguish
provider/model observations from UAR-owned signals, live or process-local state
from durable history, and estimated cost from billing authority.

#### Scenario: Operator inspects live state
- **WHEN** the runtime console receives run, approval, provider, route, cost, or protocol updates
- **THEN** the guide identifies the SSE/entity-graph path and states which state must be reloaded or is lost when the process or browser session ends

#### Scenario: Operator interprets cost
- **WHEN** a run reports usage and an estimated cost or crosses an in-process budget threshold
- **THEN** the guide labels the amount as an estimate, identifies the process/session scope, and directs billing reconciliation to the provider rather than treating the dashboard as an invoice

### Requirement: Shutdown, persistence, and recovery boundaries

The portal SHALL explain SIGINT/SIGTERM handling, run cancellation, listener
drain, registered cleanup, the configured hard deadline, emitted shutdown
outcomes, persistence-provider ownership, cold backup/restore, and embedded or
offline host responsibility. It SHALL NOT claim that an HTTP cancellation token
alone terminates every runtime resource or that a successful source check proves
backup restoration.

#### Scenario: Graceful deadline expires
- **WHEN** an HTTP/SSE request or registered cleanup remains held until the configured shutdown deadline
- **THEN** the guide states that the process exits at the deadline without the graceful-complete outcome rather than waiting indefinitely

#### Scenario: Operator restores persistent state
- **WHEN** an operator follows a local or remote database restore procedure
- **THEN** the guide requires a stopped/cold boundary where applicable, identifies the database owner and configured location, and requires functional read-back instead of treating archive creation as restore proof

### Requirement: Security and operations provenance gate

Every security and operations guide SHALL identify classified source records,
current authorities, applicable runtime profiles, state ownership, and an
explicit limitation. Local controls SHALL reject missing boundaries, unsafe
credential material, universal tenant or governance claims, missing
realtime-versus-durable distinctions, and missing shutdown or recovery limits
before accepting the complete source fixture.

#### Scenario: Security claim exceeds current authority
- **WHEN** a guide claims blanket tenant isolation, universal fail-closed Cedar enforcement, durable live history, authoritative billing, or cross-profile parity not supported by its authorities
- **THEN** local documentation validation exits non-zero and identifies the unsupported claim class

#### Scenario: Public guide contains unsafe material
- **WHEN** a guide contains raw credentials, private keys, machine-local paths, raw private-history excerpts, or internal event/session payloads
- **THEN** local documentation validation exits non-zero before the complete fixture passes
