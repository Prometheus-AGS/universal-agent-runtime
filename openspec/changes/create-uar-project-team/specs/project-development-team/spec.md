## ADDED Requirements

### Requirement: Portable project team
The project SHALL provide one validated portable team with explicit ownership, inputs, outputs, dependencies, skills, and independent review responsibilities.

#### Scenario: Work is assigned
- **WHEN** a development task is delegated
- **THEN** the coordinator selects the smallest relevant working group and assigns disjoint write paths while preserving KBD authority and dependency pins

### Requirement: Five native harness installations
The project SHALL contain native role definitions for Codex, Claude Code, Kimi Code, OpenCode, and MiniMax CLI without replacing unrelated configuration.

#### Scenario: Operator chooses a harness
- **WHEN** the operator follows the project activation instructions
- **THEN** that harness can load its project role definitions subject to documented CLI, account, and native feature prerequisites
- **AND** installation evidence distinguishes generated files, native discovery, and live execution

### Requirement: Grounded skills models and tools
The team SHALL record Rust graph limits, repository relationships, model discovery evidence, and provenance for project-local skills found through Firecrawl research.

#### Scenario: A specialist uses an external skill
- **WHEN** the specialist reads the installed skill
- **THEN** UAR pins and trusted-host/entity boundaries override conflicting examples and no example command is executed automatically

### Requirement: Independent review and handoff
The team SHALL require artifact-only adversarial review, sycophancy screening, and inspectable handoff receipts for cross-project work.

#### Scenario: Delivery is evaluated
- **WHEN** the implementation is complete
- **THEN** an independent reviewer receives requirements and final artifacts without generation history and unresolved limitations remain visible

#### Scenario: A sibling repository needs a change
- **WHEN** an interface change crosses repository ownership
- **THEN** the team writes a handoff request naming contract and evidence without assuming permission to edit or an active peer transport
