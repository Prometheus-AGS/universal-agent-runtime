## ADDED Requirements

### Requirement: A2A gRPC inherits the server host
When A2A gRPC is enabled, its bind address SHALL use the same `server.host` as HTTP and its own configured port. UAR SHALL NOT substitute a wildcard address when the configured host is loopback.

#### Scenario: Server host is loopback
- **WHEN** `server.host` is `127.0.0.1` and A2A gRPC is enabled on port 50051
- **THEN** the gRPC listener binds `127.0.0.1:50051` and no wildcard gRPC listener exists
