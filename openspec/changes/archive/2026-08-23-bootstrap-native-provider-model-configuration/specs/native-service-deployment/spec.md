## ADDED Requirements

### Requirement: Service credentials are generated from an allowlist
The bootstrap SHALL copy only approved canonical provider credential variables into the service environment, SHALL use aliases only when the canonical variable is absent, SHALL set restrictive file permissions, and SHALL never print values.

#### Scenario: Canonical and alias values both exist
- **WHEN** the source environment contains both `KIMI_API_KEY` and `KIMI_CODING_API_KEY`
- **THEN** the service environment retains the canonical `KIMI_API_KEY` value and does not expose either value in output

### Requirement: Native YAML enables both loopback listeners
The native YAML SHALL set `server.host` to `127.0.0.1`, HTTP port to 1906, and A2A gRPC port to 50051 for the server-full installed service.

#### Scenario: Native YAML is installed
- **WHEN** server-full starts from the merged native configuration
- **THEN** both configured listener ports establish at least one loopback LISTEN socket and absence of either listener is a verification failure

### Requirement: Alias mapping does not cross endpoints
The bootstrap MAY map Kimi Coding aliases to KIMI, MINIMAX_KEY to MINIMAX, and Qwen aliases to DASHSCOPE. It SHALL NOT map another endpoint's credential into MOONSHOT or ZAI.

#### Scenario: Only a nonmatching credential exists
- **WHEN** no canonical or approved alias exists for a provider
- **THEN** that provider credential and its conditional YAML seed are omitted

#### Scenario: Multiple aliases exist without the canonical name
- **WHEN** both Kimi aliases exist without `KIMI_API_KEY`, or both Qwen aliases exist without `DASHSCOPE_API_KEY`
- **THEN** `KIMI_CODING_API_KEY` wins over `KIMI_CODING_KEY`, and `QWEN_API_KEY` wins over `QWEN_TOKEN_PLAN_API_KEY`
