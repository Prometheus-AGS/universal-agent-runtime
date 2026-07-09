## ADDED Requirements

### Requirement: CI Trigger Actually Fires

A new or modified CI trigger SHALL NOT be considered verified until it is observed firing on the actual CI platform, not merely locally simulated.

#### Scenario: A workflow file exists but was never pushed

- **Given** a CI workflow file was added to the repository but never
  pushed to the branch GitHub Actions evaluates
- **When** claiming the workflow is verified
- **Then** the claim MUST be backed by an actual, observed run (via
  `gh run list` or equivalent) on the real CI platform, not just a local
  command simulation

#### Scenario: Verifying without waiting for a scheduled trigger

- **Given** a workflow's primary trigger is a `schedule` cron that may
  not fire for days
- **When** verification is needed sooner
- **Then** a manual `workflow_dispatch` run SHOULD be triggered instead
  of waiting, provided the workflow supports it
