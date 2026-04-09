## ADDED Requirements

### Requirement: Expose Prometheus metrics endpoint
The server SHALL expose `GET /metrics` returning Prometheus text exposition format.

#### Scenario: Metrics endpoint accessible
- **WHEN** a GET request is made to `/metrics`
- **THEN** the server returns HTTP 200 with Content-Type `text/plain; version=0.0.4` containing all registered metrics

#### Scenario: Metrics endpoint is unauthenticated
- **WHEN** a GET request is made to `/metrics` without authentication
- **THEN** the server returns HTTP 200 (bypasses auth middleware)

### Requirement: HTTP request metrics
The server SHALL record request count, duration, and status code for all HTTP endpoints.

#### Scenario: Request counter incremented
- **WHEN** a request to `POST /v1/chat/completions` completes with status 200
- **THEN** `uar_requests_total{method="POST",path="/v1/chat/completions",status="200"}` is incremented

#### Scenario: Duration histogram recorded
- **WHEN** a request completes
- **THEN** `uar_request_duration_seconds` histogram is updated with the request duration

### Requirement: LLM token metrics
The server SHALL record token usage per provider and model.

#### Scenario: Token usage tracked
- **WHEN** an LLM completion uses 500 input tokens and 200 output tokens on `anthropic/claude-sonnet-4-20250514`
- **THEN** `uar_llm_tokens_total{provider="anthropic",model="claude-sonnet-4-20250514",direction="input"}` increases by 500 and `direction="output"` increases by 200

### Requirement: Tool call metrics
The server SHALL record tool call count and success/failure rate.

#### Scenario: Successful tool call
- **WHEN** MCP tool `tavily__search` executes successfully
- **THEN** `uar_tool_calls_total{tool_name="tavily__search",status="success"}` is incremented

### Requirement: Active session gauge
The server SHALL maintain a gauge of active sessions.

#### Scenario: Session count reflects reality
- **WHEN** 3 chat sessions are active
- **THEN** `uar_active_sessions` reads 3
