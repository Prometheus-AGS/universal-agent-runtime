## ADDED Requirements

### Requirement: Repository-owned UI/UX Pro Max skill
The UAR repository SHALL track one canonical UI/UX Pro Max skill payload, its upstream license, and reproducibility metadata. The mandatory UI/UX routing instructions SHALL resolve agents to that tracked payload rather than relying on a machine-local installation.

#### Scenario: Fresh checkout contains the skill
- **WHEN** a developer checks out the repository without a prior local skill installation
- **THEN** the canonical UI/UX Pro Max `SKILL.md`, searchable data, scripts, references, and upstream license are present in the repository

#### Scenario: Machine-local agent state remains excluded
- **WHEN** Git ignore rules are evaluated
- **THEN** unrelated `.agents/` state remains ignored while the canonical UI/UX Pro Max skill subtree is trackable

#### Scenario: Supported tool entry points resolve
- **WHEN** a tracked tool-specific UI/UX Pro Max entry point is inspected
- **THEN** it resolves to the single canonical repository payload without duplicating the skill data

#### Scenario: Reproducibility metadata is present
- **WHEN** the repository skill installation is audited
- **THEN** tracked metadata identifies the upstream source and computed payload hash

#### Scenario: Routing instructions identify the local skill
- **WHEN** an agent follows the UI/UX work routing block and its referenced roster
- **THEN** the roster identifies the canonical local skill path and requires the agent to read its query contract before UI/UX work

#### Scenario: Installed skill is operational
- **WHEN** the skill's integrity validator and a representative stack search run from the canonical payload
- **THEN** validation succeeds and the search returns a stack-appropriate result
