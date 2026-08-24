## Purpose

Define a responsive, persistent, and functionally effective Session Configuration editor backed by explicit canonical and draft entity state.

## ADDED Requirements

### Requirement: Session configuration loads a bounded configured-model projection
Opening Session Configuration SHALL load model choices only from providers and
models configured for this UAR instance. It MUST NOT download or ingest the full
static model catalog as a side effect of opening the sheet.

#### Scenario: The sheet opens with configured providers
- **WHEN** an operator opens Session Configuration on the installed release service at `http://localhost:1906`
- **THEN** the sheet remains interactive and displays every configured provider/model choice, including configured providers absent from the static catalog
- **AND** the browser issues no request to `/api/models`

#### Scenario: The static catalog contains thousands of models
- **WHEN** the static catalog contains 7,248 or more models but the instance has a bounded configured set
- **THEN** opening the sheet publishes no more than the configured provider count plus configured model count plus six graph updates
- **AND** no publication is caused by or proportional to the static catalog size
- **AND** opening the sheet becomes interactive within two seconds on the local certification host

### Requirement: Canonical and draft session state are distinct
Committed session configuration SHALL have one canonical entity keyed by the
session identifier. Unsaved edits SHALL use a separate inspectable draft entity
keyed by session and editor identity, and field consumers SHALL subscribe only to
the values they render.

#### Scenario: An existing session is edited
- **WHEN** an operator opens the editor for a session with saved configuration
- **THEN** the canonical configuration is loaded before the draft is initialized
- **AND** the draft begins with the saved effective values without mutating canonical state

#### Scenario: An edit is cancelled
- **WHEN** an operator changes one or more draft fields and cancels or closes without saving
- **THEN** only that editor's draft is discarded
- **AND** the canonical session configuration and other editors' drafts remain unchanged

#### Scenario: One field changes
- **WHEN** an operator changes one draft field
- **THEN** subscribers to that field observe the new value
- **AND** unrelated field subscribers and the sheet shell do not receive a business-state update

### Requirement: Saved session model controls effective inference
The frontend and backend SHALL use one typed `model` field for the session model
override. Saving SHALL update persistent session configuration, and subsequent
turns for that session SHALL resolve the saved model before the agent default.

#### Scenario: A model override is saved
- **WHEN** an operator selects a configured model and saves Session Configuration
- **THEN** the request uses the backend's typed `model` field
- **AND** reopening the same session displays the persisted model
- **AND** a genuine inference turn for that session is routed through the saved model

#### Scenario: No model override is saved
- **WHEN** the session model is unset
- **THEN** inference continues to use the selected agent's default model

#### Scenario: The selected model is no longer configured
- **WHEN** a saved session model no longer resolves to an available configured provider/model
- **THEN** the editor presents an actionable unavailable state
- **AND** it does not silently substitute a different override

### Requirement: Every retained session control is effective
The Session Configuration sheet MUST NOT display or submit fields that the
backend ignores. Every retained control SHALL round-trip through the typed
session contract and affect its named runtime behavior.

#### Scenario: A context or approval control remains visible
- **WHEN** the implementation retains a context-strategy, memory, capture, scope, or tool-approval control
- **THEN** its value loads, saves, reloads, and affects the corresponding runtime policy

#### Scenario: A control has no supported runtime contract
- **WHEN** no backend and runtime behavior exists for a displayed control
- **THEN** that control is removed from the sheet rather than submitted as ignored JSON

### Requirement: Sheet controls have consistent interior spacing
The sheet header and control body SHALL use the shared design-system spacing
scale. The control body SHALL have a horizontal inset no smaller than the header
inset, a bottom inset, and standard vertical separation between logical groups at
both compact and desktop widths.

#### Scenario: Supported viewport sheets are inspected
- **WHEN** computed styles are captured at the established 320, 768, 1024, and 1440 pixel certification widths
- **THEN** the first and last controls do not touch the sheet edge
- **AND** the body horizontal inset is at least the header horizontal inset
- **AND** all controls remain visible, operable, and scrollable without overlap
