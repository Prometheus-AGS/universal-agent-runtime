## Purpose

Keeps provider keys, MCP server environment secrets and other sensitive settings out of UAR's files in readable form, keeps the decryption key out of the data directory, and tells hosts which secrets must be re-entered — without ever returning a secret value.

## ADDED Requirements

### Requirement: Sensitive settings values are never written in plaintext
The runtime SHALL NOT write a sensitive value in readable form to its settings storage, database, backups or logs. A value is sensitive when the settings schema marks its node `x-sensitive`, which SHALL include every provider `api_key`, every MCP server `env` value, and every other field the schema marks today. This SHALL hold for every write path: API writes, first-boot seeding from configuration, settings initialization and migration. A persisted sensitive value SHALL be stored either as an authenticated ciphertext bound to its setting key and field location, or as a marker with no secret content.

#### Scenario: Provider key created and updated through the API
- **WHEN** a client creates a provider with an API key through `POST /api/uar/providers` and later changes it through `PUT /api/uar/providers/{id}`
- **THEN** a byte search of every file under the data directory, the database files and the log output finds neither key value, raw or base64-encoded

#### Scenario: MCP server env secret saved through the API
- **WHEN** a client saves an MCP server definition whose `env` contains a secret value through `PUT /api/uar/mcp/servers/{name}`
- **THEN** a byte search of every file under the data directory, the database files and the log output does not find that value

#### Scenario: Configured secrets seeded at first boot
- **WHEN** UAR starts on an empty data directory with a provider key, a JWT secret and an OCR API key supplied through environment or configuration
- **THEN** none of those values is findable by byte search in the data directory after startup completes

#### Scenario: Ciphertext moved to another location
- **WHEN** a stored ciphertext is copied from one provider's row into another provider's row by direct database edit
- **THEN** the runtime does not decrypt it for the second provider and reports that provider's key as `needs_reentry`

### Requirement: Persisted secrets still work after restart
A sensitive value stored as ciphertext SHALL be decrypted at load and used exactly as the plaintext value was before this change, for as long as the same key source is available.

#### Scenario: Provider key survives restart
- **WHEN** a provider key is saved, UAR restarts with the same key source, and a request is routed to that provider
- **THEN** the upstream request carries the saved key and the provider reports `credential_state: stored`

#### Scenario: MCP env secret survives restart
- **WHEN** an MCP server with an `env` secret is saved, UAR restarts with the same key source, and the server is started
- **THEN** the MCP child process receives the secret value in its environment

### Requirement: The decryption key is never stored in the data directory
The runtime SHALL obtain the data-encryption key from, in order: an operator-supplied key in the process environment; the operating system's per-user credential store (macOS login Keychain, Windows Credential Manager, Linux Secret Service); otherwise no key. The runtime SHALL NOT write the key, or any value from which it can be derived, to the data directory, the database, configuration files, logs or any other file it controls, and SHALL NOT fall back to a key file.

#### Scenario: Data directory copied to another machine or account
- **WHEN** the complete data directory is copied and UAR starts on the copy with no access to the original key source
- **THEN** no stored secret is decrypted, every stored secret reports `needs_reentry`, and a byte search of the copy finds no key material (raw, hex or base64)

#### Scenario: Same key source after restart
- **WHEN** UAR restarts on the same data directory with the same key source
- **THEN** stored secrets are decrypted and report `stored`

#### Scenario: OS credential store available on first secret write
- **WHEN** no key exists yet, the OS credential store is available, and the first secret is saved
- **THEN** a new random 256-bit key is created in the OS credential store under an entry specific to this data store, and the secret is stored as ciphertext

### Requirement: Without a key source, secrets are not persisted
When no key source is available, the runtime SHALL accept a new secret for use by the running process only, SHALL persist the entry without the secret and marked `needs_reentry`, and SHALL report the secret as `session_only` until restart and `needs_reentry` after restart.

#### Scenario: Linux machine without a Secret Service
- **WHEN** UAR runs with no operator key and no reachable OS credential store, and a client saves a provider key
- **THEN** the provider works in the current process with `credential_state: session_only`, the key is not findable in any file, and after restart the provider reports `needs_reentry`

#### Scenario: Status reports the missing key source
- **WHEN** a client calls `GET /api/uar/secrets/status` on such a machine
- **THEN** the response reports `key_source: none` and the counts of stored and re-entry-required secrets, and contains no secret value

### Requirement: Startup migration removes existing plaintext secrets
At startup, before serving requests, the runtime SHALL find every sensitive value stored in plaintext. With a key source it SHALL replace each with ciphertext; without one it SHALL remove the value and mark the entry `needs_reentry`. For embedded storage it SHALL ensure no superseded plaintext version remains in the store's files. It SHALL log only counts (sealed, stripped, already sealed, failed), never values or key names that carry values. A second startup SHALL change nothing.

#### Scenario: Fixture with plaintext rows and a key source
- **WHEN** UAR starts with a key source on a data directory containing plaintext provider keys and MCP `env` secrets written by an earlier version
- **THEN** after startup no plaintext value is findable by byte search in any file under the data directory, the providers and MCP servers still work, and the log shows counts only

#### Scenario: Fixture with plaintext rows and no key source
- **WHEN** UAR starts without a key source on the same fixture
- **THEN** no plaintext value is findable in any file under the data directory, each affected provider reports `needs_reentry`, each affected MCP server lists the env keys needing re-entry, and the log shows counts only

#### Scenario: Migration is idempotent
- **WHEN** UAR starts a second time on a migrated data directory
- **THEN** the migration reports zero sealed and zero stripped values and does not rewrite the store

### Requirement: Secret state is visible without secret values
The runtime SHALL expose secret state without exposing any secret: providers SHALL report `credential_state` as `none`, `stored`, `session_only` or `needs_reentry`; MCP server views SHALL list env keys needing re-entry; the settings API SHALL report a state for each sensitive field; `GET /api/uar/secrets/status` SHALL report the active key source (`env`, `os_keystore` with its backend name, or `none`) and counts. No response SHALL contain a secret value or ciphertext.

#### Scenario: Provider needing re-entry
- **WHEN** a provider's stored key was stripped by migration
- **THEN** `GET /api/uar/providers/{id}` reports `credential_state: needs_reentry` and `credential_configured: false`

#### Scenario: Re-entry clears the state
- **WHEN** a client saves a new key for that provider while a key source is available
- **THEN** the provider reports `credential_state: stored` and the key works

#### Scenario: Responses never carry secrets
- **WHEN** a client reads providers, MCP servers, settings and secrets status after secrets were saved
- **THEN** no response body contains a secret value or an envelope ciphertext
