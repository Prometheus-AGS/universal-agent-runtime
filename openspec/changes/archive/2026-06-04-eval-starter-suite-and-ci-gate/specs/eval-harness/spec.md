## ADDED Requirements

### Requirement: Starter suite and two-tier CI gate

The repository SHALL ship a starter eval suite and SHALL run it in CI as a
regression gate across two tiers: a deterministic structural check on every pull
request that requires no model or API key, and a scheduled real-model run that
gates on regression against a baseline and degrades gracefully when no API key
is configured.

#### Scenario: Starter suite ships and is valid
- **WHEN** the repository is checked out
- **THEN** `evals/starter.yaml` exists, declares scorers, and loads + scores through the harness

#### Scenario: PR tier requires no key
- **WHEN** the pull-request CI runs
- **THEN** the starter suite is loaded and scored with a deterministic provider, with no API key and no model call

#### Scenario: Scheduled tier gates on regression
- **WHEN** the scheduled job runs and an API key secret is present
- **THEN** the suite runs against the real model and the job exits non-zero on regression against the baseline

#### Scenario: Scheduled tier without a key is skipped
- **WHEN** the scheduled job runs and no API key secret is present
- **THEN** the job skips the real-model run without failing
