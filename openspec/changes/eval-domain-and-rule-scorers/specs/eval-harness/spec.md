## ADDED Requirements

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
