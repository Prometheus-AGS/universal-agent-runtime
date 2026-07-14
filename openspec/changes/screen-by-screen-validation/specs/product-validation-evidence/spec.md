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
