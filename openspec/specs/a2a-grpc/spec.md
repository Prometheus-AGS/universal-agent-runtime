# A2A gRPC Specification

## Purpose

Define the A2A gRPC transport behavior and how its listener follows the server network boundary.

## Requirements

### Requirement: A2A v0.3 gRPC transport
The server SHALL expose A2A protocol methods via gRPC on a configurable port (default 50051) alongside the existing JSON-RPC HTTP endpoint.

#### Scenario: Send message via gRPC
- **WHEN** a gRPC client calls `MessageSend` with a valid A2A task message
- **THEN** the server processes it identically to the JSON-RPC `message/send` method and returns the task state

#### Scenario: Get task via gRPC
- **WHEN** a gRPC client calls `TaskGet` with a valid task ID
- **THEN** the server returns the current task state

#### Scenario: Streaming responses
- **WHEN** a gRPC client calls `MessageStream` (server-streaming RPC)
- **THEN** the server streams task state updates as the agent processes the request

### Requirement: gRPC and HTTP share handler logic
The gRPC transport SHALL delegate to the same handler functions as the JSON-RPC endpoint.

#### Scenario: Identical behavior
- **WHEN** the same A2A request is sent via gRPC and JSON-RPC
- **THEN** both produce identical task states and artifacts

### Requirement: A2A gRPC inherits the server host
When A2A gRPC is enabled, its bind address SHALL use the same `server.host` as HTTP and its own configured port. UAR SHALL NOT substitute a wildcard address when the configured host is loopback.

#### Scenario: Server host is loopback
- **WHEN** `server.host` is `127.0.0.1` and A2A gRPC is enabled on port 50051
- **THEN** the gRPC listener binds `127.0.0.1:50051` and no wildcard gRPC listener exists
