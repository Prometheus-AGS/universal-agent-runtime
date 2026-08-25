# agent-ui-design-workflow Specification

## Purpose

Define the repository-wide quality workflow and skill precedence that agents follow when designing, evaluating, refactoring, or refining user interfaces.

## Requirements

### Requirement: UI design skills follow repository precedence
Repository agent guidance SHALL direct user-interface design, evaluation,
refactoring, and implementation through the installed project skills in the
operator-defined order: Impeccable first, Anthropic `frontend-design` second,
and UI/UX Pro Max third.

#### Scenario: An agent begins user-interface design work
- **WHEN** an agent evaluates, refactors, designs, or implements a user interface
- **THEN** the agent applies Impeccable first for evaluation, refactoring, and initial design
- **AND** applies Anthropic `frontend-design` second for general agentic UI/UX design
- **AND** applies UI/UX Pro Max third for remaining general design guidance

#### Scenario: Managed routing guidance is regenerated
- **WHEN** an auto-managed UI/UX routing block is refreshed
- **THEN** the project-owned skill precedence remains outside the managed region
- **AND** both `AGENTS.md` and the `CLAUDE.md` symlink continue to expose the same order

### Requirement: UI design decisions receive independent critique
Repository agent guidance SHALL make dual-agent Impeccable critique followed by
fresh-context adversarial review the standard quality gate for initial UI
ideation, evaluation, refactoring, and refinement.

#### Scenario: A UI direction or implementation is evaluated
- **WHEN** an agent evaluates an initial design direction, refactor, or refinement
- **THEN** two isolated subagents perform the Impeccable critique assessments
- **AND** a fresh-context adversarial reviewer examines the resulting artifact or implementation diff
- **AND** blocking findings are resolved before the direction is accepted
