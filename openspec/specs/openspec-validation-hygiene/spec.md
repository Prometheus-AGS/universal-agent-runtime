# openspec-validation-hygiene Specification

## Purpose
TBD - created by archiving change openspec-global-validation-cleanup. Update Purpose after archive.
## Requirements
### Requirement: Active Change Delta Hygiene

Active OpenSpec changes used by KBD phase closure SHALL keep requirement deltas valid under strict OpenSpec validation before they are used as archive-gate evidence.

#### Scenario: Repair invalid normative wording

- **Given** an active OpenSpec change contains a requirement delta that fails because it lacks `SHALL` or `MUST`
- **When** the delta still represents intended product behavior
- **Then** the requirement text MUST be repaired to use parser-compatible normative wording without changing the intended behavior

#### Scenario: Retire stale invalid deltas

- **Given** an active OpenSpec change contains an invalid requirement delta that no longer represents intended behavior
- **When** the KBD executor evaluates the cleanup path
- **Then** the stale delta MUST be removed, archived, or otherwise retired through an auditable OpenSpec workflow record

### Requirement: Validation Evidence Recording

KBD phase closure SHALL record targeted and active-change OpenSpec validation evidence when validation hygiene cleanup is performed.

#### Scenario: Validation passes

- **Given** the cleanup has repaired or retired invalid deltas
- **When** targeted and active-change validation pass
- **Then** KBD progress MUST record the commands, pass status, and verification timestamp

#### Scenario: Validation remains blocked

- **Given** validation still fails after the cleanup
- **When** the failure belongs to another active change or unrelated spec surface
- **Then** KBD progress MUST record the blocker, failing command, and concise failure summary

### Requirement: Runtime Behavior Preservation

OpenSpec validation hygiene cleanup SHALL avoid runtime behavior changes unless an explicit product requirement requires implementation work.

#### Scenario: Spec-only cleanup

- **Given** the cleanup only repairs OpenSpec requirement wording and KBD evidence
- **When** the change is applied
- **Then** no frontend, backend, provider-routing, realtime event, or credential configuration files MUST be modified

#### Scenario: Provider credentials

- **Given** validation cleanup runs during provider-related hardening
- **When** the cleanup verifies OpenSpec state
- **Then** provider credentials MUST NOT be written into repository files or echoed into persistent logs

