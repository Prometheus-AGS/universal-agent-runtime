## ADDED Requirements

### Requirement: Suites declare their scorers

An eval suite SHALL be able to declare the scorers applied to its cases, and the
runner SHALL build scorers from that declaration. When a suite declares no
scorers, the system SHALL fall back to a default scorer set so existing suites
behave unchanged.

#### Scenario: Declared scorers are used
- **WHEN** a suite declares a list of scorers (e.g. `json_valid`, `pattern_match`)
- **THEN** the runner scores each case with exactly those scorers

#### Scenario: No declaration falls back to the default set
- **WHEN** a suite declares no scorers
- **THEN** the runner applies the default set (non-empty + sycophancy, plus exact-match + contains when every case has an expected output)

#### Scenario: Existing suites deserialize unchanged
- **WHEN** a suite file without a `scorers` field is loaded
- **THEN** it loads successfully and uses the default scorer set
