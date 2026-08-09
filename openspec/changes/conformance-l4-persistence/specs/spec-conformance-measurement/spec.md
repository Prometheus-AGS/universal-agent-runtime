# spec-conformance-measurement

## ADDED Requirements

### Requirement: Capabilities defined by durability are measured across a restart

A capability whose defining property is surviving a restart MUST be measured by
a write, a shutdown, a reboot against the same store, and a read. A shape
assertion against such a capability is vacuous: it establishes that an endpoint
returns well-formed JSON while leaving untested the one property that defines
the capability.

The measurement harness MUST therefore be able to shut a server down
deterministically and reopen the same persistence path. Where the runtime
already implements graceful shutdown, the harness MUST use that path rather than
killing the process, so the measurement exercises the production shutdown
sequence.

#### Scenario: Persisted state survives a restart

- **GIVEN** a server booted against a fixed persistence path
- **WHEN** a resource is written through the public API, the server is shut down
  through its graceful-shutdown path, and a new server is booted against the
  same path
- **THEN** reading the resource returns what was written
- **AND** the case is labelled `l4_`

#### Scenario: The round-trip can fail

- **GIVEN** an L4 round-trip case
- **WHEN** the second boot is pointed at a different persistence path
- **THEN** the case FAILS
- **AND** this negative control is recorded, because a round-trip that passes
  against an empty store proves nothing

#### Scenario: Durability cannot be measured

- **GIVEN** a capability whose round-trip is structurally impossible in this
  harness
- **WHEN** the matrix is assembled
- **THEN** it is published as an exclusion naming the blocking condition
- **AND** it is NOT recorded as a pass on the strength of a shape assertion

### Requirement: Test-owned shutdown does not weaken signal-driven shutdown

Exposing a caller-supplied shutdown trigger MUST be additive. Signal-driven
shutdown — the path a deployed runtime actually takes — MUST continue to work
unchanged when no caller-supplied trigger is present.

#### Scenario: A deployed runtime receives SIGTERM

- **GIVEN** a server started without a caller-supplied cancellation trigger
- **WHEN** the process receives SIGTERM
- **THEN** the ingestion pool drains and in-flight connections are drained
  exactly as before this change
