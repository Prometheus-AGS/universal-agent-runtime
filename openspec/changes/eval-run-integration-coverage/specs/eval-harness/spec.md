## ADDED Requirements

### Requirement: End-to-end run pipeline is covered by an automated test

The system SHALL cover the eval run pipeline (load a suite, run each case
through a completion provider, score, summarize, persist, and compare to a
baseline) with an automated test that uses a deterministic recorded provider, so
the pipeline is verifiable without a live model.

#### Scenario: Deterministic pipeline run
- **WHEN** a suite is run through the pipeline with a recorded provider
- **THEN** each case produces scored results, the summary reflects the recorded outputs, and results persist and reload unchanged

#### Scenario: Regression verdicts are exercised
- **WHEN** the summary is compared to baselines representing no-baseline, equal, and a drop beyond the threshold
- **THEN** the comparison reports clean, clean, and regressed respectively

#### Scenario: Provider failure is contained
- **WHEN** the provider has no recorded output for a case
- **THEN** that case yields a contained failure result and the run still completes
