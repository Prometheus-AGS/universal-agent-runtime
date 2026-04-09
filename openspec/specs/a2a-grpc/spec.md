## ADDED Requirements

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
