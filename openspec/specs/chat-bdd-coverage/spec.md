# chat-bdd-coverage Specification

## Purpose
TBD - created by archiving change bdd-chat-scenario-suite. Update Purpose after archive.
## Requirements
### Requirement: No-KB Chat Scenario Coverage
The BDD suite SHALL prove that a chat conversation with no knowledge base
attached completes end-to-end against the real running app and stub LLM,
with no retrieval-related UI state rendered.

#### Scenario: Plain chat with no knowledge base
- **WHEN** a user starts a new conversation with no knowledge base attached
  and sends a message
- **THEN** the assistant's response is rendered in the transcript and no
  knowledge-base/citation UI element appears

### Requirement: Knowledge-Base-Influenced Chat Scenario Coverage
The BDD suite SHALL prove that enabling a knowledge base and asking a
question whose answer depends on ingested content produces a response that
actually reflects the retrieved content, not a generic answer.

#### Scenario: Retrieval-influenced response
- **WHEN** a user ingests a fixture document containing a distinctive phrase,
  enables that knowledge base for the conversation, and asks a question only
  answerable from that phrase
- **THEN** the rendered response contains content derived from the ingested
  phrase and does not contain the suite's "missing context" marker

### Requirement: Skill Activation Chat Scenario Coverage
The BDD suite SHALL prove that a message triggering skill activation causes
the frontend to visibly indicate the activated skill during the response.

#### Scenario: Skill visibly activates mid-conversation
- **WHEN** a user sends a message that matches a configured skill's
  activation trigger
- **THEN** the transcript shows a visible skill-activation indicator
  associated with that response

### Requirement: Tool Call Chat Scenario Coverage
The BDD suite SHALL prove that a message triggering a tool call causes the
frontend to render both the tool invocation and its result in the
transcript, exercising the real tool-call rendering path.

#### Scenario: Tool call invoked and result surfaced
- **WHEN** a user sends a message that triggers a tool call against the stub
  LLM's tool-call fixture
- **THEN** the transcript shows a tool-call block for the invoked tool
  followed by its result content

### Requirement: Agent Switching Chat Scenario Coverage
The BDD suite SHALL prove that switching the active agent mid-session
changes which agent answers subsequent messages, not just that the selector
UI updates.

#### Scenario: Switching agent changes the answering agent
- **WHEN** a user sends a message, switches the active agent via the agent
  selector, and sends a second message
- **THEN** the second response is attributable to the newly selected agent
  (distinct fixture/model identity from the first response)

### Requirement: Provider/Model Routing Chat Scenario Coverage
The BDD suite SHALL prove that provider/model configuration changes affect
which model actually answers a chat message, not only which model is
displayed as selected.

#### Scenario: Model configuration changes the answering model
- **WHEN** a user configures the conversation to use a specific model bound
  to a distinct stub fixture, then sends a message
- **THEN** the rendered response is attributable to that specific model's
  fixture, not the default model's fixture

### Requirement: Scenario Registry Documentation
The project SHALL maintain a checked-in registry documenting every BDD chat
scenario, its `.feature` file location, and its current pass/fail status.

#### Scenario: Registry reflects suite state
- **WHEN** `docs/BDD_SCENARIOS.md` is read after a suite run
- **THEN** it lists all six chat scenarios with their `.feature` file paths
  and a status matching the most recent suite run

### Requirement: Video-Proof Evidence Capture
Each BDD chat scenario run SHALL produce video evidence packaged per the
project's `bdd-video-proof` convention (MP4 remux, SHA-256 manifest keyed to
commit).

#### Scenario: Scenario run produces video evidence
- **WHEN** the BDD chat suite completes a run
- **THEN** an MP4 recording and a SHA-256 manifest entry exist for each
  executed scenario under the certification bundle path

