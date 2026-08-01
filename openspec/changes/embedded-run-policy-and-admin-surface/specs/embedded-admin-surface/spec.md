# embedded-admin-surface

## ADDED Requirements

### Requirement: Embedded runtime exposes typed settings administration

The SDK `Runtime` SHALL expose methods to read and write typed settings backed by
the runtime's own `SettingsManager` and persistence, so an embedded host can read
and write settings (including `run_policy.global`) with no HTTP service.

#### Scenario: Read and write the global run policy setting

- **WHEN** a host calls `set_setting("run_policy.global", <value>)` and later
  `get_setting("run_policy.global")`
- **THEN** the returned value equals the value written
- **AND** the next resolved run uses that global policy

#### Scenario: Settings snapshot returns registered settings and their types

- **WHEN** a host calls `settings_snapshot()`
- **THEN** it returns the current settings values and the registered setting types
  (the same shape the service admin exposes)

### Requirement: Embedded runtime exposes agent administration

The SDK `Runtime` SHALL expose agent CRUD (list, get, upsert, delete) delegating to
the same persistence-backed agent store the service path uses, so an embedded host
can manage agent definitions in-process.

#### Scenario: Upsert then list an agent

- **WHEN** a host calls `upsert_agent(<artifact>)` and then `list_agents()`
- **THEN** the returned list contains the upserted agent with its provider/model
  policy intact

#### Scenario: Delete an agent

- **WHEN** a host calls `delete_agent(id)` for a previously upserted agent
- **THEN** a subsequent `get_agent(id)` returns none

#### Scenario: Embedded agent admin does not require an HTTP service

- **WHEN** the runtime is built in embedded mode with no HTTP server
- **THEN** all settings and agent administration methods succeed against the
  in-process persistence
