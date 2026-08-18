## ADDED Requirements

### Requirement: Every product screen is functionally validated with recorded evidence
Each shipped screen SHALL have a validated purpose/function record backed by a
live browser test run with video evidence, covering the operator's functional
matrix (agents, skills, RAG, memory, auth, isolation, local-first).

#### Scenario: Orchestrator answers live with proof
- **WHEN** the validation suite asks the orchestrator and default agents defined questions in the browser
- **THEN** expected answers stream back and the run is captured on video in the certification bundle

#### Scenario: Admin screen validation
- **WHEN** the suite exercises an admin screen's primary function (e.g. create KB, disable skill, mint API key)
- **THEN** the function succeeds end-to-end and the screen's record links its video segment

#### Scenario: An observed screen defect blocks validation
- **WHEN** a live primary-function scenario exposes a supported product defect
- **THEN** the defect is recorded and repaired at the smallest bounded product
  surface before that scenario may be reported as passing
