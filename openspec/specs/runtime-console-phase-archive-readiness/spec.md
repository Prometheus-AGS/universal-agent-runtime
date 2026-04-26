# runtime-console-phase-archive-readiness Specification

## Purpose
TBD - created by archiving change runtime-console-archive-readiness. Update Purpose after archive.
## Requirements
### Requirement: Final Phase Gate Evidence

Runtime console phase archive readiness SHALL record final validation evidence before the phase is closed.

#### Scenario: Final gate passes

- **Given** all dependent validation-hardening changes are archived
- **When** the final gate commands pass
- **Then** KBD progress MUST record the commands, pass statuses, and verification timestamp

#### Scenario: Final gate fails

- **Given** a final gate command fails
- **When** the failure prevents safe archive
- **Then** KBD progress MUST record the failing command and concise blocker summary

### Requirement: Runtime Console Change Archive

The canonical runtime console OpenSpec change SHALL be archived only after its validation checklist is complete.

#### Scenario: Runtime console change is complete

- **Given** `runtime-console-entity-workflow` has all validation tasks complete
- **When** final phase gate validation passes
- **Then** the change MUST be archived through OpenSpec and synced to main specs

#### Scenario: Runtime console change remains blocked

- **Given** `runtime-console-entity-workflow` has an incomplete validation task or blocker
- **When** archive readiness is evaluated
- **Then** the blocker MUST be recorded instead of archiving the change

### Requirement: Generated Asset Cleanliness

Archive readiness SHALL leave generated frontend asset hash churn out of the tracked worktree unless it is intentionally part of the change.

#### Scenario: Backend tests rewrite static index

- **Given** a backend build script rewrites `static/index.html`
- **When** the rewritten asset references are not part of the validated change
- **Then** the generated static index churn MUST be restored before archive evidence is recorded

