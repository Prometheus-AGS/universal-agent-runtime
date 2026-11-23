## ADDED Requirements

### Requirement: Every persisted setting has an explicit application class
UAR SHALL classify every persisted setting key by how its saved value is applied: `live`, `next_turn`, `restart` or `inert`. The class SHALL come from one table that names the reader justifying each entry. The settings API SHALL return the class as `apply` on every setting row. A key with no table entry SHALL be treated as `restart`, never as `live` or `next_turn`.

#### Scenario: Every seeded key is classified
- **WHEN** the settings API lists all settings of a freshly bootstrapped instance
- **THEN** every row carries an `apply` value of `live`, `next_turn`, `restart` or `inert`
- **AND** no seeded key resolves its class through the unclassified default

#### Scenario: A key without an entry is classified conservatively
- **WHEN** a setting key that has no table entry is read through the settings API
- **THEN** its `apply` value is `restart`
- **AND** its status never reports `effective` unless a restart has loaded the saved value

### Requirement: A saved value is reported effective only when a reader applied it
The settings API SHALL report `application_status` per row as `effective`, `pending`, `restart_required` or `not_applied`, and SHALL report `effective` only after the reader of the value has applied it. A `restart` key reports `restart_required` until the process restarts. An `inert` key reports `not_applied` before and after a restart.


#### Scenario: A boot-only key is saved
- **WHEN** an operator saves `rag.chunk_size` with a value different from the one loaded at boot
- **THEN** the response row reports `application_status: "restart_required"`
- **AND** `effective` still holds the boot value while `saved` holds the new value

#### Scenario: A boot-only key after restart
- **WHEN** the instance restarts after that save
- **THEN** the same row reports `application_status: "effective"` and `effective` equals `saved`

#### Scenario: A per-turn key is saved
- **WHEN** an operator saves `prompt_caching.enabled`
- **THEN** the response row reports `application_status: "effective"`
- **AND** the next request reads the new value

#### Scenario: An inert key is saved
- **WHEN** a client saves `sandbox.<key>` through the settings API
- **THEN** the write persists
- **AND** the row reports `apply: "inert"` and `application_status: "not_applied"` before and after a restart

#### Scenario: A hot-swap that did not happen is not reported as applied
- **WHEN** an embedding setting is saved and the live embedding swap is skipped or fails
- **THEN** the row reports `restart_required`, not `effective`

### Requirement: The default provider is changed only through the live route
`llm.default_provider` SHALL be written only by `POST /api/uar/providers/{id}/default`, which records it applied only after the registry swap succeeds. The settings API SHALL reject any write, bulk write or reset targeting it with 422 `unsupported_setting` and a message naming that route, and SHALL persist nothing. Reads continue. The key is classified `live`.

#### Scenario: The live route succeeds
- **WHEN** an operator sets provider B as default through the default route
- **THEN** the registry default is B for the next request
- **AND** the `llm.default_provider` row reports `application_status: "effective"`

#### Scenario: A settings write names another provider
- **WHEN** a client submits `PUT /api/uar/settings/llm.default_provider` naming provider C
- **THEN** the response is 422 with `error: "unsupported_setting"` and a message naming the default route
- **AND** the stored row and the registry default are unchanged

#### Scenario: A bulk write includes the key
- **WHEN** a client submits a bulk `llm` namespace update that includes `default_provider` alongside valid keys
- **THEN** the request is rejected with 422 and none of its keys is persisted

#### Scenario: The route names an unknown provider
- **WHEN** a client calls the default route for a provider that is not registered
- **THEN** the response is 404 and neither the row nor the registry changes

### Requirement: Mutation activation reports derive from the same classification
The `activation` field of settings mutation responses SHALL be derived from the application class, not from a separate list of keys. For every saved key whose class is `restart` the response SHALL report `restart_required`; for `inert` keys it SHALL report `not_applied`.

#### Scenario: A bulk save mixes classes
- **WHEN** a bulk save contains `rag.chunk_size` (`restart`) and `resilience.retry_max_attempts` (`next_turn`)
- **THEN** `activation` marks `rag.chunk_size` as `restart_required`
- **AND** it does not mark `resilience.retry_max_attempts`

### Requirement: Settings schemas mark controls that must not render
Settings type schemas SHALL support a field-level `x-uar-ui-hidden` marker with an `x-uar-ui-reason` of `yaml-only`, `not-read` or `managed-by-registry`. Every property of a namespace removed from the console, `llm.model`, `llm.default_provider` and `rag.embedding_provider` / `rag.embedding_model` SHALL carry the marker. The marker SHALL NOT delete the key, unregister the type, change YAML handling, or change the REST read path.

#### Scenario: The schema of a removed namespace is read
- **WHEN** a client reads the `sandbox` settings type
- **THEN** every property carries `x-uar-ui-hidden: true`
- **AND** the type remains registered and its rows remain readable

#### Scenario: A retained key is read
- **WHEN** a client reads the `resilience` settings type
- **THEN** no property carries `x-uar-ui-hidden`

### Requirement: Stored rows of removed namespaces remain inert by default
The values stored in the settings table for removed namespaces SHALL remain after this change. UAR SHALL NOT delete, rewrite or migrate them, and SHALL NOT let a stored value change runtime behavior. Deleting them requires a separate operator-approved migration that touches no other key.

#### Scenario: An instance upgrades with stored rows
- **WHEN** an instance with edited `server.*` rows upgrades and starts
- **THEN** the rows are unchanged
- **AND** the listener binds from the YAML value exactly as before
