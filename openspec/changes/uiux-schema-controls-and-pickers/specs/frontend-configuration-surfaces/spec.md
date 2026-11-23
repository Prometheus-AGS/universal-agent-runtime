## ADDED Requirements

### Requirement: Schema-driven settings render the declared control kind
The generic schema panel SHALL choose each field's control by its `x-control` kind
first, then by an explicit namespace-and-key binding for provider and model fields,
then by JSON type. A kind the panel does not know SHALL fall back to the JSON-type
control and never to a raw text field for a non-string type.

#### Scenario: A slider is declared
- **WHEN** a property declares `x-control: slider` with `x-slider-min` and `x-slider-max`
- **THEN** it renders a slider bounded to those values with the current value shown as text

#### Scenario: A tag input is declared
- **WHEN** a string-array property declares `x-control: tag-input`
- **THEN** it renders as a list of removable chips with an add control and no comma-separated text field

#### Scenario: An unknown kind is declared
- **WHEN** a property declares an `x-control` kind the panel does not implement
- **THEN** it renders the control for its JSON type and the field stays editable and saveable

### Requirement: Array, object and numeric settings use structured controls
The panel SHALL render a string array as chips, an array with an `enum` as a
multi-select, an object with declared `properties` as grouped fields, and a number as a
bounded slider or stepper with its unit. A raw JSON editor SHALL appear only for an
object with no declared properties, inside an "Advanced" disclosure.

#### Scenario: A string array is edited
- **WHEN** an operator adds and removes values in an array field such as `native_tools.web_fetch_allowed_domains`
- **THEN** the saved value is an array of the remaining strings
- **AND** no value is split from a comma-separated string

#### Scenario: A bounded number is edited
- **WHEN** a numeric property has `minimum` and `maximum`
- **THEN** the control prevents values outside the range and shows the unit when the schema or title states one

#### Scenario: An object without properties is shown
- **WHEN** an object property declares no `properties`
- **THEN** its raw JSON editor is inside a collapsed "Advanced" disclosure and reports invalid JSON before save

#### Scenario: A resilience or governance number is edited
- **WHEN** an operator edits a retry count, delay or budget field in the resilience or governance panels
- **THEN** the control is bounded and unit-labeled like the schema panel's number control
- **AND** an out-of-range value cannot be saved

### Requirement: Provider and model choices use one picker family
Every control that chooses an LLM provider or model SHALL be built from one picker
family: a Base UI Combobox composition grouped by provider, searchable, keyboard
operable, with a visible label and a status for loading, error and empty. The existing
`ModelSelector` and `SettingModelPicker` SHALL be replaced by it. The existing rule that
fewer than eight options use a simple select SHALL be preserved.

#### Scenario: A model is chosen in a session, agent, skill or team binding
- **WHEN** an operator opens the model control in the session configuration panel, the agent editor, the skill editor or the team binding step
- **THEN** the same picker family opens with models grouped by provider and a search input
- **AND** selecting a model stores its `provider/model` route exactly as before

#### Scenario: A stored model is no longer available
- **WHEN** the stored value is not among the picker's options
- **THEN** the control shows the stored value marked unavailable with an invalid state
- **AND** it does not clear or replace the value until the operator chooses another

#### Scenario: Fewer than eight options exist
- **WHEN** a provider has between one and seven enabled models
- **THEN** the control behaves as a simple select with every enabled model available

### Requirement: Provider and model fields do not accept typed values
A settings or page field that stores a provider id or a model id SHALL accept only a
value chosen from the picker's options. This covers `llm.embedding.model`, the
knowledge-base default embedding provider and model, the memory embedding model, and
the credentials provider id. Fields in namespaces removed from the console by the
no-reader decision are out of scope.

#### Scenario: An embedding model is chosen
- **WHEN** an operator edits `llm.embedding.model` or the memory embedding model
- **THEN** a picker lists the options for the selected embedding provider
- **AND** typing a value that is not an option selects nothing

#### Scenario: A credentials provider is chosen
- **WHEN** an operator creates a credential
- **THEN** the provider is chosen from the configured and catalog providers
- **AND** rotating an existing credential shows its provider read-only

#### Scenario: No option source is available for a field
- **WHEN** a provider or model field has no option source
- **THEN** the control shows the stored value read-only with a stated reason
- **AND** it does not fall back to a free-text input

### Requirement: Large option sets stay responsive
A picker with more than about 1,000 options SHALL render only the visible rows and SHALL
open without a main-thread task longer than 200 ms. Filtering SHALL be case-insensitive
over display name and raw identifier.

#### Scenario: The full catalog is searched
- **WHEN** an operator opens a picker backed by the full catalog of about 8,000 models and types a query
- **THEN** the list narrows to matching display names and raw identifiers
- **AND** a no-match state appears when nothing matches

#### Scenario: The list is navigated by keyboard
- **WHEN** an operator moves through a virtualized list with the arrow keys
- **THEN** the highlighted option is always scrolled into view and announced to assistive technology

### Requirement: Enumerable settings values are never free text
A control whose valid values can be listed SHALL be a picker, a toggle or a select. Free
text SHALL be limited to prompts, content, names, secrets, URLs, identifiers an operator
creates, search filters and server paths without a source. Each retained free-text control
SHALL appear in an inventory fixed before implementation.

#### Scenario: The audited surfaces are swept
- **WHEN** the end-to-end sweep enumerates every text input and textarea on the audited settings, providers, credentials, skills, memory, agents, teams and presentations surfaces
- **THEN** each textbox matches exactly one entry of the fixed inventory by surface and accessible name
- **AND** a control with no entry fails the sweep and names the control

#### Scenario: An inventory entry has no control
- **WHEN** an inventory entry matches no control on its surface
- **THEN** the sweep fails as stale and names the entry

#### Scenario: A multi-value field is edited
- **WHEN** an operator edits agent allowed tools, skill preferred tools, skill keywords or retryable HTTP status codes
- **THEN** allowed tools and preferred tools are chosen from the tool catalog, status codes from the standard status list, and keywords entered as chips
- **AND** no value is parsed from a comma-separated string

### Requirement: Skill import does not require a typed server path
The skill import dialog SHALL let the operator choose a skill with a browser file
chooser and SHALL NOT require typing an absolute server path. Import authority SHALL stay
with the trusted host: the dialog SHALL NOT call an import endpoint whose authority class
has not been confirmed.

#### Scenario: A SKILL.md is chosen
- **WHEN** an operator chooses a valid `SKILL.md` file
- **THEN** the dialog shows its name, description and version before import
- **AND** confirming imports it and the skill appears in the list without a typed path

#### Scenario: A SKILL.md is invalid
- **WHEN** the chosen file fails validation
- **THEN** the dialog shows the server's reason and imports nothing

#### Scenario: The skill name already exists
- **WHEN** the imported skill's id conflicts with an existing skill
- **THEN** the dialog reports the conflict and the existing skill is unchanged
