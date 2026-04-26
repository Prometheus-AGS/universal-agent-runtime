# runtime-console Specification

## Purpose
TBD - created by archiving change runtime-console-entity-workflow. Update Purpose after archive.
## Requirements
### Requirement: Shared Workflow State
UAR SHALL coordinate runtime-console planning state through OpenSpec and KBD
across Codex, Claude Code, Cursor, and OpenCode.

#### Scenario: Tool starts work on the runtime console
- **WHEN** an agent tool begins work on the runtime console
- **THEN** it MUST read `openspec/config.yaml`, the active OpenSpec change, and
  `.kbd-orchestrator/current-waypoint.json` before planning or implementation.

#### Scenario: Phase progress changes
- **WHEN** an agent completes a runtime-console task
- **THEN** it MUST update `.kbd-orchestrator/phases/runtime-console-ux/progress.json`
  with the task status and source tool.

### Requirement: Surreal Memory Workflow Mirror
UAR SHALL expose Surreal Memory MCP as a secondary mirror for workflow state.

#### Scenario: Workflow state is persisted
- **WHEN** KBD workflow state changes
- **THEN** the state SHOULD be mirrored to the `surreal_memory` MCP server with
  project, phase, task, timestamp, and `source_tool` metadata.

#### Scenario: Workflow state conflicts
- **WHEN** file state and memory mirror state disagree
- **THEN** `.kbd-orchestrator/` MUST remain authoritative unless a human
  explicitly promotes the memory state.

### Requirement: Runtime Entity Graph
The frontend SHALL model live runtime activity as normalized graph entities.

#### Scenario: Runtime event arrives
- **WHEN** SSE, AG-UI, A2UI, provider health, route decision, approval, artifact,
  or memory events arrive
- **THEN** the frontend MUST normalize them into runtime entity types before UI
  components render them.

#### Scenario: Runtime entity updates
- **WHEN** an existing runtime entity is updated
- **THEN** every visible console surface that reads that entity MUST update
  without manual refresh.

### Requirement: Runtime Console UX
The frontend SHALL provide a compact runtime operations console inspired by
librefang's registry/detail structure.

#### Scenario: Operator opens admin console
- **WHEN** the operator opens `/admin`
- **THEN** the default surface MUST be the runtime cockpit rather than a static
  provider onboarding page.

#### Scenario: Operator searches console surfaces
- **WHEN** the operator presses Cmd+K or Ctrl+K
- **THEN** the console MUST open a command search dialog that navigates to
  runtime, provider, protocol, memory, skills, tools, and settings surfaces.

### Requirement: Protocol Compatibility Console
The frontend SHALL expose protocol and provider compatibility status in the
runtime console.

#### Scenario: Operator inspects protocols
- **WHEN** the operator opens the protocols surface
- **THEN** it MUST show Anthropic REST, OpenAI-compatible REST, AG-UI, A2UI, MCP,
  and `liter-llm` routing status areas.

#### Scenario: Model routing decision is available
- **WHEN** a model routing decision is present in the entity graph
- **THEN** the console MUST show the selected provider/model and the routing
  reason when available.

