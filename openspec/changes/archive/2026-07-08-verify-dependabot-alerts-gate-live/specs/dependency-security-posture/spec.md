## MODIFIED Requirements

### Requirement: CI Trigger Actually Fires

A new or modified CI trigger SHALL NOT be considered verified until it is observed firing on the actual CI platform, not merely locally simulated. When a job's correctness depends on a credential's runtime scope (not just its presence), verification MUST confirm the credential actually has that scope on the real platform, not just that a local credential with different provenance produced the expected result.

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

#### Scenario: A job's correctness depends on a secret's runtime scope

- **Given** a CI job authenticates with a repo secret (e.g.
  `secrets.SUBMODULES_TOKEN`) whose actual permission scope was not
  independently confirmed before being reused for a new purpose
- **When** a local dry-run used a different credential (e.g. an
  interactive developer token) to validate the job's logic
- **Then** the job MUST still be run for real in CI with the actual
  secret before being considered verified, since a local dry-run with a
  different credential does not confirm the real secret's scope is
  sufficient
