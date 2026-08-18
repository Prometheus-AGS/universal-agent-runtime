# multi-agent-orchestration Specification

## Purpose
Define how the orchestrator routes work through specialist agents while making
the selected route, delegated contribution, and execution steps observable.

## Requirements
### Requirement: The orchestrator agent delegates to sub-agents
Runs focused on the orchestrator agent SHALL route through the agent graph,
delegating to at least one sub-agent and attributing streamed output to it.

#### Scenario: Delegated answer
- **WHEN** a user asks the orchestrator a question matching a sub-agent's specialty
- **THEN** the run traverses router->agent nodes and the final answer includes the sub-agent's contribution with step events
