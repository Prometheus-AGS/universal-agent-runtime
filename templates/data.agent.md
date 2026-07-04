# Agent: Data Analyst

## Metadata
```yaml
version: "1.0"
description: "RAG-backed analyst that answers questions against a knowledge base, decomposing multi-part queries and citing sources."
author: "Prometheus AGS"
tags: ["data", "rag", "template", "v2"]
```

## Identity
```yaml
name: "data-analyst"
role: "assistant"
persona: "A meticulous analyst who grounds every claim in retrieved evidence."
system_prompt: "You are a data analyst. Answer only from retrieved context where possible, decompose multi-part questions, and cite which source supported each claim."
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
sources:
  - id: "primary-kb"
    type: "vector"
retrieval:
  strategy: "hybrid"
  top_k: 8
```

## Memory Model
```yaml
conversation:
  enabled: true
  max_turns: 200
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
max_iterations: 20
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
needs_reasoning: true
needs_structured_output: true
min_context: 200000
```

## Prompt Dialect
```yaml
wants_reasoning: true
hard: false
```

## RAG Configuration
```yaml
enabled: true
decomposition: true
verification: true
audit: true
knowledge_base_ids: ["primary-kb"]
```

## Context Strategy
```yaml
type: "hierarchical"
short_term_turns: 10
mid_term_summary_tokens: 2000
long_term_facts_tokens: 500
```

## API Harness
```yaml
protocols: ["a2a", "rest"]
stream_mode: "openai"
```
