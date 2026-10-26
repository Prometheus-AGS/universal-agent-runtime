## ADDED Requirements

### Requirement: Agent definitions persist effective prompt behavior
Users SHALL be able to create, read, update and remove authorized custom agent definitions, and prompting a saved definition SHALL use its persisted prompt and configuration after a true process restart on each supported durable adapter.

#### Scenario: Custom prompt survives restart
- **WHEN** a user saves an agent with a distinctive harmless instruction and restarts UAR over the same durable store
- **THEN** the saved definition remains available and a genuine response demonstrates the saved instruction is effective

### Requirement: Default and orchestrator agents cannot be deleted
The `default-agent` and `orchestrator-agent` definitions SHALL be protected from deletion through every supported public management path. Refused deletion SHALL preserve both durable definitions and live availability. Recreating a deleted definition during startup SHALL NOT satisfy deletion protection.

#### Scenario: Built-in deletion is refused without mutation
- **WHEN** an authorized user attempts to delete either protected built-in agent
- **THEN** the operation explicitly refuses, persisted definitions remain unchanged, and both agents remain discoverable and promptable immediately and after restart

#### Scenario: Custom deletion remains usable
- **WHEN** an authorized user deletes an unprotected custom agent
- **THEN** the custom definition is removed under its normal ownership rules without disabling or deleting the protected built-ins
