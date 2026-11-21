## MODIFIED Requirements

### Requirement: Settings behavior remains compatible through decomposition
The decomposition SHALL preserve, for every namespace the console still offers, its namespace key, default active panel, metadata-based availability, generic namespace fallback, responsive layout, visible controls and copy, loading/error/saved states, validation, save/reload actions, JWT gating, provider/model semantics, and realtime settings updates. Navigation grouping and order are governed by the runtime-console-ux requirement for settings areas, and the removal of namespaces and controls that no code reads is governed by "Settings that no code reads are absent from the console"; this requirement does not freeze either. It MUST NOT remove or retype an existing REST payload field, change persistence, authentication, AG-UI/A2UI, or entity schemas; additive fields on settings responses and schemas are permitted.

#### Scenario: Operator selects a settings namespace
- **WHEN** the operator selects any available custom or schema-driven settings item that the console still offers
- **THEN** the same panel, controls, values, and action semantics render as before decomposition, except for controls removed under "Settings that no code reads are absent from the console"

#### Scenario: Settings metadata is unavailable or incomplete
- **WHEN** type metadata is empty or an available namespace has no custom panel
- **THEN** the existing availability and generic-schema fallback behavior remains intact

#### Scenario: User settings authentication is evaluated
- **WHEN** the configured frontend key is or is not a JWT
- **THEN** the user-settings panel preserves its existing gated state and remote save/reload behavior

### Requirement: Settings namespace reads use canonical backend slugs
The settings frontend SHALL translate internal namespace keys to the backend's canonical URL slug before every namespace read. The translation MUST preserve the existing response decoding and non-success error propagation, and MUST NOT change save routes, payloads, persistence, provider configuration, or realtime state.

#### Scenario: Provider namespace is read
- **WHEN** the frontend reads the internal `provider` namespace
- **THEN** it requests `/api/uar/settings/providers`
- **AND** it does not request `/api/uar/settings/provider`

#### Scenario: Underscored namespace is read
- **WHEN** the frontend reads the internal `intent_classifier` namespace
- **THEN** it requests `/api/uar/settings/intent-classifier`
- **AND** it does not issue an underscored settings namespace request

#### Scenario: Canonical namespace is unchanged
- **WHEN** the frontend reads the internal `resilience` namespace
- **THEN** it requests `/api/uar/settings/resilience`

#### Scenario: Canonical settings read fails
- **WHEN** a canonical settings namespace response is non-successful
- **THEN** the existing status-based error propagation is preserved
- **AND** the UI does not report a successful load

#### Scenario: Installed runtime settings are inspected
- **WHEN** the production static bundle is served by the installed runtime on port 1906 and an operator opens Provider Overrides and Intent Classifier
- **THEN** the configured provider records render and Intent Classifier loads through its hyphenated route
- **AND** the browser emits no singular, underscored, or other settings namespace 404

## ADDED Requirements

### Requirement: Settings that no code reads are absent from the console
The settings console SHALL NOT offer a control for a setting whose saved value no code reads, and that removal SHALL NOT depend on type metadata having loaded. The removed set is the 14 namespaces named in the scenarios, `llm.model`, and any key the application-class table marks `inert`. YAML keys, boot readers and stored database rows SHALL remain.


#### Scenario: Settings navigation is opened
- **WHEN** an operator opens Settings
- **THEN** none of `server`, `security`, `persistence`, `skill_evolution`, `llm_failover`, `context_management`, `file_processing`, `sycophancy`, `sandbox`, `vision`, `acp`, `kreuzberg`, `unstructured` or `mistral_ocr` appears in the navigation
- **AND** every retained namespace appears

#### Scenario: Schema-driven panels render
- **WHEN** a schema-driven panel renders a namespace whose schema marks properties `x-uar-ui-hidden`
- **THEN** those properties are not rendered

#### Scenario: Type metadata fails to load
- **WHEN** the settings type request fails or returns no types
- **THEN** the removed namespaces are still absent from the navigation

#### Scenario: A retained namespace contains a removed field
- **WHEN** an operator opens the `llm` panel
- **THEN** it shows no Default Model control
- **AND** every other control in the panel maps to a key that a reader consumes

#### Scenario: A removed key is still honored from YAML
- **WHEN** an operator sets a removed namespace's value in YAML and restarts
- **THEN** the boot reader uses the YAML value
- **AND** the console offers no control for it

### Requirement: The system default model is not editable as a settings field
The settings console SHALL NOT offer `llm.default_provider` as an editable field. The system default model SHALL be changed only through the registry default provider and that provider's default model, using the live default route. The console SHALL NOT present a settings field that takes effect only at restart as a system model control.

#### Scenario: The llm panel is opened
- **WHEN** an operator opens the `llm` settings panel
- **THEN** it shows no Default LLM Provider ID control

#### Scenario: The operator changes the system default
- **WHEN** an operator sets a configured provider as the default from the Providers page
- **THEN** the change calls the live default route
- **AND** the next request resolves to the new default without a restart

### Requirement: The console displays what each saved setting actually does
The console SHALL display the application status of every setting it offers, read through a typed domain hook exported by `@/platform/entities`. A `restart_required` setting SHALL show the text "Saved. Takes effect after restart." beside its control. Status SHALL NOT rely on colour alone. The frontend settings type SHALL match the flattened fields the API returns.


#### Scenario: A boot-only setting is saved
- **WHEN** an operator saves a `restart`-class setting
- **THEN** the control shows "Saved. Takes effect after restart."
- **AND** the save summary names the key and its status

#### Scenario: A per-turn setting is saved
- **WHEN** an operator saves a `next_turn` setting
- **THEN** no restart message appears for it

#### Scenario: One control is edited
- **WHEN** an operator edits one control in a namespace panel
- **THEN** each status indicator subscribes to one key's status and the indicators of the other controls do not rerender

#### Scenario: Components reach status without store imports
- **WHEN** imports of the settings status UI are inspected
- **THEN** it imports the status hook from `@/platform/entities`
- **AND** it imports no graph store, service or API module directly

### Requirement: Each record type has one editing surface
Records owned by a first-class admin page SHALL be edited on that page only. A Settings entry that edits the same records SHALL be replaced by a link to the owning page once every control it offered exists there. The embedding model choice SHALL be offered once, as `llm.embedding.*`.


#### Scenario: The agent or skill settings entry is opened
- **WHEN** an operator opens the `agent_config` or `skill_config` entry in Settings
- **THEN** it shows a link to the Agents page or the Skills page respectively
- **AND** the same control is available and saves on the Agents page

#### Scenario: Parity is missing
- **WHEN** the owning page lacks a control that the Settings entry offered
- **THEN** the Settings entry remains until the control exists on the owning page

#### Scenario: Embedding is configured
- **WHEN** an operator looks for the embedding model in Settings
- **THEN** only the `llm.embedding.*` controls offer it
- **AND** `rag.embedding_provider` and `rag.embedding_model` are not offered

### Requirement: No settings control writes a key the backend rejects
A settings panel SHALL NOT render a control bound to a key that the backend rejects on every write path. A namespace save SHALL NOT be able to lose valid edits because it includes such a key.

#### Scenario: RAG settings are co-edited
- **WHEN** an operator changes Chunk Size, Chunking Strategy and Semantic Threshold together and saves
- **THEN** all three values persist and read back
- **AND** the save reports each as restart-required

#### Scenario: A client writes a rejected key
- **WHEN** a client submits a bulk update that includes `rag.embedding_model`
- **THEN** the backend rejects the request and persists none of its other keys

### Requirement: A secret setting never leaves the server in clear text
Every settings read (single key, namespace, list, drift list) and every mutation response SHALL return a stored secret only as a mask of the stored length. Saving an unchanged mask SHALL keep the stored secret; saving a new value SHALL replace it. A secret value SHALL NOT appear in any settings response or in the instance logs.

#### Scenario: A secret is saved and read back
- **WHEN** an operator saves a provider API key and then reads the provider settings
- **THEN** the response holds only mask characters, one per stored character

#### Scenario: An unrelated field is saved with the mask unchanged
- **WHEN** the operator changes another field and submits the API key as the mask returned by the previous read
- **THEN** the next provider request uses the original secret

#### Scenario: A secret is replaced
- **WHEN** the operator saves a different API key
- **THEN** the next provider request uses the new secret and the read-back mask has the new length

#### Scenario: A mask is submitted for a missing row
- **WHEN** a client submits a mask placeholder for a setting row that does not exist
- **THEN** the response is 404 and nothing is created

#### Scenario: Secrets and drift
- **WHEN** a drift list entry holds a secret
- **THEN** its stored and configured values appear only as masks, and no response or log line contains the secret

### Requirement: The console shows how the runtime booted
The console SHALL read `GET /api/uar/settings/boot-status` and, when the configuration mode is not `primary`, SHALL show a persistent banner on every console page naming the rejected configuration file and its failed digest prefix, with a link to the Configuration status panel. The panel SHALL list ignored embedding values and skipped provider rows by key and reason. The console SHALL NOT display configuration values or parser text.

#### Scenario: The runtime started from the last known good configuration
- **WHEN** the console loads after a boot that fell back
- **THEN** the banner names the rejected file and the first 12 characters of its digest
- **AND** the banner does not contain any text from the rejected file

#### Scenario: A normal boot
- **WHEN** the configuration mode is `primary` and nothing was ignored or skipped
- **THEN** no banner is shown

#### Scenario: The boot status request fails
- **WHEN** the boot status request fails
- **THEN** the panel shows an error state and no success or "normal boot" text

### Requirement: The console shows configuration drift and resolves it in one action
The Configuration status panel SHALL list every drift entry returned by `GET /api/uar/settings/drift` for keys whose application class is not `inert`, showing for each key the saved value and the configured value (masked when secret) and, for provider drift, the differing field names only. Each resolvable entry SHALL offer one action that calls `POST /api/uar/settings/drift/{key}/resolve` with `use_config`. The action SHALL NOT be offered for `llm.default_provider` or `governance.*`.

#### Scenario: A value differs from YAML
- **WHEN** an operator edited a setting and the YAML later differs
- **THEN** the panel lists the key with both values

#### Scenario: Drift is resolved
- **WHEN** the operator activates the action for an entry
- **THEN** the stored value equals the YAML value and the entry leaves the list

#### Scenario: The entry is already resolved
- **WHEN** the resolve request returns 404
- **THEN** the panel says the drift no longer exists and does not report that it resolved a value

#### Scenario: The default provider drifts
- **WHEN** `llm.default_provider` differs from YAML
- **THEN** the entry offers no resolve action and points to the Providers default control

### Requirement: The console shows what a configuration reload did
The Configuration status panel SHALL offer a reload action that calls `POST /api/uar/settings/reload` and SHALL show the returned status and the lists `applied`, `retained_with_drift`, `restart_required` and `skipped` with keys and reasons only. A rejected reload SHALL be shown as rejected with its failure class and SHALL NOT be shown as a success.

#### Scenario: A reload changes live and boot-only keys
- **WHEN** the YAML edit changed a per-turn key and a boot-only key and the operator reloads
- **THEN** the per-turn key is listed under `applied`
- **AND** the boot-only key is listed under `restart_required`

#### Scenario: A reload is rejected
- **WHEN** the YAML is invalid and the operator reloads
- **THEN** the panel shows the rejected state and the failure class
- **AND** no success text appears and the stored values are unchanged

