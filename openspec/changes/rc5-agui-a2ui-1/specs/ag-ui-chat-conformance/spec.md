## ADDED Requirements

### Requirement: Dedicated AG-UI 1.0 transport

UAR SHALL validate the dedicated AG-UI route against the final official 1.0 wire contract for the supported event and request types. UAR SHALL retain the existing admission, interruption, and replay safety guarantees on this route.

#### Scenario: A client receives a governed and replayable run

- **WHEN** a client starts an admitted run or attaches to its retained stream
- **THEN** the producer declares protocolVersion 1.0 on RUN_STARTED
- **AND** supported events match the official schema and lifecycle ordering
- **AND** replay projects retained events without admitting execution again
