## ADDED Requirements

### Requirement: Eval results are persisted to files

The system SHALL persist a run's `EvalResult`s to a JSON file under a results
directory, named by suite and timestamp, so runs are retained for comparison.

#### Scenario: Results written
- **WHEN** a run's results are saved for suite `s`
- **THEN** a JSON file `<dir>/s-<timestamp>.json` is written containing those results, re-loadable into `EvalResult`s

### Requirement: Per-scorer summary and baseline

The system SHALL compute a per-scorer mean summary over a run, and SHALL store
and load a named baseline summary per suite.

#### Scenario: Summary is per-scorer mean
- **WHEN** a run has multiple cases each scored by scorer `X`
- **THEN** the summary's value for `X` is the mean of those scores

#### Scenario: Baseline round-trips
- **WHEN** a baseline summary is saved and loaded for a suite
- **THEN** the loaded summary equals the saved one; loading a missing baseline yields none

### Requirement: Delta-vs-baseline regression detection

The system SHALL flag a scorer as regressed when its current mean drops below the
baseline mean by more than a configured threshold, and SHALL roll up to an
overall regressed flag. With no baseline, there SHALL be no regression.

#### Scenario: Regression detected
- **WHEN** a scorer's current mean is below its baseline mean by more than the threshold
- **THEN** that scorer is marked regressed and the report's overall flag is true

#### Scenario: Within threshold is not a regression
- **WHEN** a scorer's current mean is equal to or above the baseline (or below by ≤ threshold)
- **THEN** it is not marked regressed

#### Scenario: No baseline
- **WHEN** there is no baseline for the suite
- **THEN** the report shows no regressions (the run can establish a baseline)
