# multi-agent-orchestration Specification

## Purpose
Define how the orchestrator routes work through specialist agents while making
the selected route, delegated contribution, and execution steps observable.

## Requirements
### Requirement: The orchestrator agent delegates to sub-agents
Runs focused on the orchestrator agent SHALL route through the agent graph,
delegating to at least one sub-agent that executes as a persisted child thread with its own artifact, skills, tools, and intersected policy, and attributing streamed output to it through lifecycle events rather than text prefixes.

#### Scenario: Delegated answer
- **WHEN** a user asks the orchestrator a question matching a sub-agent's specialty
- **THEN** the run traverses router->agent nodes, the sub-agent runs as a child thread, and the final answer includes the sub-agent's contribution with live step and lifecycle events

#### Scenario: Sub-agent uses its tools
- **WHEN** the delegated sub-agent's artifact declares tools within the parent's policy
- **THEN** the sub-agent can call them and its tool events carry the child's id and canonical path
