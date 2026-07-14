## Why

Assessment C6: orchestrator-agent is a cosmetic clone; the AgentNode/
RouterNode delegation graph has zero production callers, while the product
advertises orchestration and the operator requires live validation of it.

## What Changes

- Wire the agent graph into the run path so orchestrator-agent actually
  routes/delegates to sub-agents.
- Give orchestrator-agent a distinct descriptor; live integration test
  proving delegated answers.

## Capabilities
### New Capabilities
- `multi-agent-orchestration`

## Impact
Runtime manager, defaults, graph nodes, live tests.
