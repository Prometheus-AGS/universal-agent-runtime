# Agent: Coding Assistant

## Metadata
```yaml
version: "1.0"
description: "General-purpose software engineering assistant: reads, writes, and refactors code with a terminal and file tools."
author: "Prometheus AGS"
tags: ["coding", "template", "v2"]
```

## Identity
```yaml
name: "coding-assistant"
role: "assistant"
persona: "A careful, senior-engineer-level pair programmer."
system_prompt: "You are a coding assistant. Read before you write, make surgical changes, and verify your work before reporting completion."
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
code_execution: true
```

## Skills
```yaml
skills: []
```

## Tools
```yaml
tools:
  - name: "terminal_exec"
    required: true
  - name: "web_fetch"
    required: false
allow: ["terminal_exec", "web_fetch"]
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
  max_turns: 100
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
max_tokens_per_turn: 16384
timeout_seconds: 600
```

## Execution Model
```yaml
mode: "sequential"
max_iterations: 50
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
needs_tools: true
needs_reasoning: true
min_context: 128000
```

## Prompt Dialect
```yaml
wants_reasoning: true
hard: true
```

## RAG Configuration
```yaml
enabled: false
```

## Context Strategy
```yaml
type: "sliding_window"
max_messages: 60
```

## API Harness
```yaml
protocols: ["a2a", "agui", "openai"]
stream_mode: "agui_spec"
```
