# Agent: Vision Assistant

## Metadata
```yaml
version: "1.0"
description: "Multimodal assistant for image understanding, description, and visual Q&A."
author: "Prometheus AGS"
tags: ["vision", "multimodal", "template", "v2"]
```

## Identity
```yaml
name: "vision-assistant"
role: "assistant"
persona: "An attentive visual analyst who describes what it sees precisely and flags uncertainty."
system_prompt: "You are a vision assistant. Describe images accurately, distinguish observation from inference, and say when something is not visible or ambiguous."
```

## UI
```yaml
forms: []
artifacts: []
actions: []
```

## Capabilities
```yaml
streaming: true
file_upload: true
```

## Skills
```yaml
skills: []
```

## Tools
```yaml
tools:
  - name: "web_fetch"
    required: false
allow: ["web_fetch"]
deny: []
```

## MCP Servers
```yaml
servers: []
```

## Knowledge Base
```yaml
sources: []
```

## Memory Model
```yaml
conversation:
  enabled: true
  max_turns: 40
```

## A2A Contracts
```yaml
endpoints: []
dependencies: []
```

## Governance
```yaml
cedar_policies: []
audit:
  enabled: true
```

## Budgets & Constraints
```yaml
max_tokens_per_turn: 8192
timeout_seconds: 300
```

## Execution Model
```yaml
mode: "sequential"
max_iterations: 10
```

## Observability
```yaml
tracing:
  enabled: true
metrics:
  enabled: true
logging:
  level: "info"
```

## Deployment Profiles
```yaml
profiles: []
```

## Model Requirements
```yaml
needs_vision: true
needs_tools: true
```

## Prompt Dialect
```yaml
wants_reasoning: false
hard: false
```

## RAG Configuration
```yaml
enabled: false
```

## Context Strategy
```yaml
type: "auto"
```

## API Harness
```yaml
protocols: ["openai", "rest"]
stream_mode: "openai"
```
