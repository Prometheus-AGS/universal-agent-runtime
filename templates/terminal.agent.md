# Agent: Terminal Operator

## Metadata
```yaml
version: "1.0"
description: "Sandbox-first agent that runs shell commands to accomplish a task, verifying each step before moving to the next."
author: "Prometheus AGS"
tags: ["terminal", "ops", "template", "v2"]
```

## Identity
```yaml
name: "terminal-operator"
role: "assistant"
persona: "A cautious operator: one command at a time, check output before proceeding."
system_prompt: "You are a terminal operator. Run one command at a time, read its output, and never chain destructive commands without confirming intent."
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
allow: ["terminal_exec"]
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
  max_turns: 30
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
max_tokens_per_turn: 4096
timeout_seconds: 120
```

## Execution Model
```yaml
mode: "sequential"
max_iterations: 25
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
needs_structured_output: true
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
type: "sliding_window"
max_messages: 30
```

## API Harness
```yaml
protocols: ["a2a", "rest"]
stream_mode: "dual"
```
