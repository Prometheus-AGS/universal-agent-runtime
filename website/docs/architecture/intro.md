# Architecture

Universal Agent Runtime (UAR) is a Rust/Axum process that owns model routing,
agent execution, governed tool calls, retrieval, memory, and normalized runtime
events. The React 19 operator interface reaches those capabilities through typed
REST and SSE services; it never calls providers or persistence directly.

```mermaid
flowchart LR
    UI[React operator interface] -->|REST + SSE| UAR[UAR server-full]
    SDK[Rust, Python, and TypeScript SDKs] -->|HTTP + SSE| UAR
    UAR --> LLM[Model providers]
    UAR --> MCP[MCP and native tools]
    UAR --> DB[(SurrealDB)]
    UAR --> A2A[A2A peers]
    UAR --> AGUI[AG-UI event consumers]
    UAR --> A2UI[A2UI renderers]
```

## Runtime request flow

```mermaid
sequenceDiagram
    participant Client
    participant API as Axum API
    participant Runtime as Run manager
    participant Policy as Cedar policy
    participant Provider as Model provider

    Client->>API: Start run
    API->>Runtime: Create governed execution
    Runtime->>Policy: Authorize action or tool
    Policy-->>Runtime: Allow, require approval, or deny
    Runtime->>Provider: Stream completion
    Provider-->>Runtime: Text and tool deltas
    Runtime-->>Client: Normalized AG-UI events
```

## Prometheus platform boundary

UAR is one service in the wider Prometheus platform. It owns inference and
agent execution; the surrounding services retain their own security and data
responsibilities.

```mermaid
flowchart TB
    Client[Client or operator] --> Gate[Flint Gate\nedge authentication]
    Gate --> UAR[Universal Agent Runtime\ninference and governed execution]
    UAR --> Fabric[Flint Realtime Fabric\ndurable event distribution]
    Forge[Flint Forge\nRLS data APIs and edge execution] --> Fabric
    Admin[Flint Platform Agent\nauthenticated administration] --> Gate
    Admin --> Forge
```

SurrealDB is the Stable server authority. PGlite is a browser or desktop cache
for local threads and messages; versioned server events reconcile the frontend
entity graph. AG-UI defines the event vocabulary. A2UI accepts validated
declarative artifacts from an approved component catalog and does not execute
model-provided HTML or JavaScript.

Read the [deployment](../deployment), [security](../security), and
[API reference](../api-reference) guides for the operational boundaries.
