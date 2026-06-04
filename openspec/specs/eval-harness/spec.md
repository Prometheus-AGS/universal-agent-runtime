# eval-harness Specification

## Purpose
TBD - created by archiving change eval-domain-and-rule-scorers. Update Purpose after archive.
## Requirements
### Requirement: Eval domain model

The system SHALL provide a typed, serializable eval domain: an eval case (id +
input + optional expected output + metadata), an eval suite (name + cases), a
score (scorer name + value in 0.0–1.0 + optional detail), and an eval result
(suite + case id + model + scores + timestamp).

#### Scenario: Domain round-trips
- **WHEN** an eval result is serialized and deserialized
- **THEN** its case id, scores, and metadata are preserved unchanged

### Requirement: Scorer contract

The system SHALL define a `Scorer` contract that maps a `(case, output)` pair to
a normalized `Score` whose value is always within 0.0–1.0. Scoring SHALL be
deterministic for rule-based scorers and SHALL NOT perform IO.

#### Scenario: Score is normalized
- **WHEN** any built-in scorer scores an output
- **THEN** the returned value is between 0.0 and 1.0 inclusive

### Requirement: Built-in rule-based scorers

The system SHALL provide rule-based scorers: exact-match and contains (against
the case's expected output), JSON-validity, non-empty, a regex/pattern match,
and a sycophancy scorer (higher value = less sycophantic) derived from the
existing sycophancy detector.

#### Scenario: Exact match
- **WHEN** the output equals the case's expected output
- **THEN** the exact-match scorer returns 1.0, otherwise 0.0

#### Scenario: Contains
- **WHEN** the output contains the expected substring
- **THEN** the contains scorer returns 1.0, otherwise 0.0

#### Scenario: JSON validity
- **WHEN** the output parses as valid JSON
- **THEN** the json-valid scorer returns 1.0, otherwise 0.0

#### Scenario: Sycophancy scorer
- **WHEN** a clean (non-sycophantic) output is scored
- **THEN** the sycophancy scorer returns a high value (≈1.0), and a flagged output returns a lower value

### Requirement: Eval suites load from golden files

The system SHALL load an `EvalSuite` from a file path, supporting JSON and YAML
by extension. A malformed or missing file SHALL produce an error (not a panic).

#### Scenario: Load a JSON suite
- **WHEN** a `.json` suite file with cases is loaded
- **THEN** an `EvalSuite` with those cases is returned

#### Scenario: Load a YAML suite
- **WHEN** a `.yaml`/`.yml` suite file with cases is loaded
- **THEN** an `EvalSuite` with those cases is returned

#### Scenario: Missing/malformed file errors
- **WHEN** the path does not exist or the content does not parse
- **THEN** the loader returns an error and does not panic

### Requirement: Runner executes cases through a completion provider and scores them

The system SHALL run a suite by, for each case, obtaining an output from a
pluggable completion provider and applying the configured scorers, producing one
`EvalResult` per case (with a timestamp). The runner SHALL NOT depend on a
specific LLM client (it depends on the provider abstraction), so it is testable
without a live model.

#### Scenario: Cases scored into results
- **WHEN** a suite of N cases is run with a provider and a set of scorers
- **THEN** N `EvalResult`s are produced, each containing one score per scorer

#### Scenario: Per-case completion error is contained
- **WHEN** the provider returns an error for a case
- **THEN** that case yields a failed result (a completion score of 0.0 with the error detail) and the remaining cases still run

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

### Requirement: Eval harness is runnable via a CLI subcommand

The binary SHALL expose an `eval` subcommand to run and inspect suites, while
preserving the default (no-subcommand) behavior of starting the server.

#### Scenario: Default invocation runs the server
- **WHEN** the binary is run with no subcommand
- **THEN** it starts the server exactly as before this change

#### Scenario: Run a suite
- **WHEN** `eval run <suite>` is invoked
- **THEN** the suite is loaded, each case is completed via the orchestrator and scored, results are persisted, the run is compared to the baseline, and a per-scorer report is printed

#### Scenario: Regression sets a non-zero exit code
- **WHEN** `eval run <suite>` detects a regression against the baseline (without `--update-baseline`)
- **THEN** the process exits with a non-zero status (a CI gate), otherwise it exits zero

#### Scenario: Update baseline
- **WHEN** `eval run <suite> --update-baseline` is invoked
- **THEN** the run's summary is saved as the suite's baseline and the process exits zero

#### Scenario: Inspect baseline / list results
- **WHEN** `eval baseline <suite>` or `eval list` is invoked
- **THEN** the stored baseline summary, or the result files, are printed

