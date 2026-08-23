# Skill Hot Reload Specification

## Purpose

Define how skill changes become available without restarting the server while preserving the behavior of already-active agent runs.

## Requirements

### Requirement: Skills can be reloaded without server restart
The skill registry SHALL support hot-reloading skills from the skills directory or database without requiring a server restart.

#### Scenario: New skill file added
- **WHEN** a new skill YAML file is added to the skills directory
- **THEN** the skill is registered within 30 seconds (via filesystem watcher) or immediately via API call

#### Scenario: Skill updated via API
- **WHEN** a PUT request updates an existing skill's definition
- **THEN** the skill registry reflects the change immediately for subsequent requests

#### Scenario: Skill removed
- **WHEN** a skill file is deleted from the skills directory
- **THEN** the skill is deregistered and no longer matched in subsequent requests

### Requirement: Hot-reload does not disrupt active runs
Active agent runs SHALL continue using the skill version that was bound at run start.

#### Scenario: Mid-run skill update
- **WHEN** a skill is updated while an agent run is using it
- **THEN** the active run continues with the original skill definition
