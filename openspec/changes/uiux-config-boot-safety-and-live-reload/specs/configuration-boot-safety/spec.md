## ADDED Requirements

### Requirement: Last-known-good configuration holds validated source bytes only
After a start from the primary configuration file reaches readiness, UAR SHALL save that file's exact bytes and their SHA-256 digest to an owner-only directory by atomic replace. UAR SHALL NOT save environment, command-line, defaulted or vault-resolved values, or a serialized configuration.

#### Scenario: Saved copy equals the source file and contains no environment secret
- **WHEN** the server starts from `config.yaml` with secrets supplied only through environment variables and reaches readiness
- **THEN** the directory holds a copy whose bytes and SHA-256 equal those of `config.yaml`, its permissions are owner-only, and no environment-supplied secret appears in any file under the directory

#### Scenario: Nothing to save without a configuration file
- **WHEN** the server starts with no resolvable configuration file
- **THEN** no last-known-good file is created and boot status reports that no copy is present

#### Scenario: Torn copy is not used
- **WHEN** the saved bytes do not match the digest in their record
- **THEN** the copy is treated as absent

### Requirement: Saving the last-known-good copy never blocks or overwrites a start
A failure to save SHALL be reported by class and SHALL NOT stop the start. A process that started from the saved copy SHALL NOT save over it.

#### Scenario: Save failure does not stop the start
- **WHEN** the directory is not writable at the readiness point
- **THEN** the server still reaches readiness and boot status reports the save failure by class

#### Scenario: A fallback start does not re-save
- **WHEN** the server started from the saved copy and reaches readiness
- **THEN** the saved copy and its digest record are unchanged

### Requirement: Boot falls back only for an unreadable or unparseable configuration source
When the primary file exists but cannot be read or parsed into the configuration model, a server run SHALL start from the saved copy only if a second load with the copy substituted, under the same command line and environment, passes every validation and is not a remote profile. Strict mode, a disabled fallback and one-shot commands SHALL fail with the original error.

#### Scenario: Broken YAML starts from the saved copy
- **WHEN** a saved copy exists, `config.yaml` is replaced by invalid YAML, and the server is started
- **THEN** the server reaches readiness serving the copy's values and boot status reports the saved-copy mode with the digest of the broken file

#### Scenario: Missing file does not fall back
- **WHEN** the configured file path does not exist
- **THEN** startup fails with the original error and the copy is not used

#### Scenario: Remote profile never falls back
- **WHEN** the second load resolves to the remote deployment profile
- **THEN** startup fails with the original error

#### Scenario: Strict mode never falls back
- **WHEN** strict configuration mode is set and the file is invalid
- **THEN** startup fails with the original error

### Requirement: A fallback start is visible and carries no value text
A start from the saved copy SHALL be reported in the log and in boot status with the failure class, the failed file's path and digest, and the copy's digest and saved time. The report SHALL NOT include a parsed value or raw parser text.

#### Scenario: Report contains digests and no file content
- **WHEN** the server starts from the saved copy after a parse failure
- **THEN** the log line and the boot status body contain the class, the failed file's path and both digests and no substring of the broken file

#### Scenario: Fallback cleared by a valid reload
- **WHEN** the server started from the saved copy and the primary file is then corrected and reloaded successfully
- **THEN** boot status reports that the fallback was cleared by reload and the saved copy is unchanged

### Requirement: A validation refusal never falls back
A configuration that parses and then fails any validation, security refusal, secret requirement or vault step SHALL fail startup and SHALL NOT start from the saved copy or alter it. Refusals SHALL be recognized by processing stage, not by message text.

#### Scenario: Tightened security that fails validation does not boot an older config
- **WHEN** a saved copy exists, the configuration is edited so that JWT authentication is required without a deliberate signing secret, and the server is started
- **THEN** startup fails before any listener is bound, the refusal is logged, no fallback is reported, and the saved copy is byte-identical to before

#### Scenario: Same edit with a deliberate secret boots normally
- **WHEN** the same edit is started with a deliberate signing secret supplied through the environment
- **THEN** the server starts from the primary file

### Requirement: Malformed persisted values are skipped individually
A persisted `llm.embedding.*` value that cannot be decoded or is out of range, and a persisted `provider.{id}` row that cannot be decoded, SHALL be skipped individually at startup. The configuration-built value or seeded provider stays in effect for the skipped item, other items apply, and the stored row is not changed or deleted.

#### Scenario: Malformed embedding dimension does not stop startup
- **WHEN** the stored `llm.embedding.vector_dimension` is not a number and the server starts
- **THEN** the server reaches readiness and the stored row is unchanged

#### Scenario: One malformed provider row does not drop the others
- **WHEN** one stored provider row cannot be decoded and the server starts
- **THEN** every other stored provider, including operator-edited ones, is registered and the bad row is unchanged

#### Scenario: A malformed governance preference is not tolerated away
- **WHEN** the stored `governance.enabled` is not a boolean and the server starts
- **THEN** the existing fail-closed behavior applies

### Requirement: Skipped persisted values are reported by key and reason
Each skipped persisted value SHALL be listed in boot status and in the log by key and reason class, without the stored value.

#### Scenario: Skipped items are named without values
- **WHEN** a malformed embedding value and a malformed provider row were skipped at start
- **THEN** boot status lists each key with its reason class and contains neither stored value

### Requirement: Reload reconciles configuration-owned settings and never overrides an operator edit
After a successful reload UAR SHALL reconcile settings and provider rows with the same per-key decision as startup: configuration-owned rows follow the new configuration, operator-edited rows keep their value and are reported as drift. A reload SHALL NOT delete a row or reconcile a `governance.*` key.

#### Scenario: Operator edit survives reload and restart
- **WHEN** an operator changes a provider's default model through the provider API and the YAML later sets a different default model
- **THEN** after a reload and again after a restart the stored default model is the operator's and the provider is listed as drifted with the field name

#### Scenario: Untouched configuration-owned row follows the YAML
- **WHEN** the YAML changes a provider no operator has edited and the configuration is reloaded
- **THEN** the stored row reflects the YAML value

### Requirement: A rejected reload performs no write
A reload rejected by parsing, validation or strict mode SHALL leave settings, provider rows and the live configuration unchanged and SHALL report the rejection by class.

#### Scenario: Invalid YAML is rejected without writes
- **WHEN** the YAML is invalid and a reload is requested
- **THEN** the request is rejected, no setting or provider row changes, and the previous configuration remains in effect

### Requirement: Drift is visible with both values
UAR SHALL list drift with the stored value, the configuration value, the detection time and whether startup or reload detected it. Values SHALL be masked as other settings reads are masked, and provider drift SHALL list field names only.

#### Scenario: Drift lists both values
- **WHEN** an operator-edited setting differs from the configuration after a reload
- **THEN** the drift listing shows the stored and configuration values, the detection time and `reload`

### Requirement: Drift can be resolved by adopting the configuration value
An administrator SHALL be able to resolve one drifted key by adopting its configuration value, stored as configuration-owned. Resolving `llm.default_provider` or a `governance.*` key SHALL be refused, and resolving a key with no drift SHALL be reported as not found.

#### Scenario: Adopting the configuration value
- **WHEN** an administrator resolves a drifted setting with the configuration value
- **THEN** the stored value equals the configuration value, the key is no longer listed, and a later YAML change reaches the row on reload

#### Scenario: Default provider drift is not resolvable here
- **WHEN** an administrator resolves drift on `llm.default_provider`
- **THEN** the request is refused and the stored default is unchanged

### Requirement: A reload reports what it did
A reload SHALL return, by key and without values, the settings applied, retained with drift, requiring a restart, not applied, and skipped with a reason. A restart-class key, and a YAML-at-boot key edited in the YAML, SHALL be reported as requiring a restart; a key with no reader SHALL be reported as not applied, using the settings change's classification. The legacy well-known reload route SHALL keep its response body and perform the same reconcile.

#### Scenario: Restart-class, YAML-at-boot and no-reader keys are reported honestly
- **WHEN** a reload changes `memory.max_context_tokens`, `server.port`, `vision.auto_detect` and `resilience.requests_per_second` in the YAML
- **THEN** the result lists `resilience.requests_per_second` as applied, `memory.max_context_tokens` and `server.port` as requiring a restart and `vision.auto_detect` as not applied, and the listener port is unchanged

#### Scenario: Same key edited in the database is not applied
- **WHEN** `server.port` is edited through the settings API instead
- **THEN** the setting reads back with status `not_applied`

#### Scenario: Legacy route now has an effect
- **WHEN** the well-known reload route is called after a YAML change to a next-turn setting
- **THEN** the stored setting reflects the change and the response body is the same schema document as before

### Requirement: Drift, reload and resolve responses never expose a secret
Every response and log line of the drift, reload and resolve operations SHALL mask values of sensitive settings exactly as settings reads do, and SHALL NOT contain the stored or configured secret.

#### Scenario: Sensitive drift is masked
- **WHEN** an operator-set secret setting differs from its YAML value after a reload
- **THEN** the drift listing shows the masked form for both values and neither secret appears in the drift, reload or resolve responses or in process output
