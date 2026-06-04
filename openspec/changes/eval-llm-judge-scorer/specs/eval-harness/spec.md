## ADDED Requirements

### Requirement: LLM-as-judge scorer

The system SHALL provide an LLM-as-judge scorer that grades a candidate output
against a rubric via a completion provider and returns a normalized score. The
scorer SHALL parse the model's verdict deterministically and SHALL NOT panic on
malformed output. Judge scores SHALL be advisory — reported and persisted, but
not part of the hard regression gate.

#### Scenario: Rubric grading
- **WHEN** a suite declares an `llm_judge` scorer with a rubric
- **THEN** each case's input + candidate output are sent to the provider with the rubric, and the parsed score is recorded

#### Scenario: Deterministic verdict parse
- **WHEN** the judge response contains a JSON object `{ "score": <0.0–1.0>, "reason": <text> }`
- **THEN** the score is extracted and clamped to 0.0–1.0

#### Scenario: Malformed verdict is contained
- **WHEN** the judge response cannot be parsed into a verdict
- **THEN** the scorer returns 0.0 with a detail explaining the failure, and does not panic
