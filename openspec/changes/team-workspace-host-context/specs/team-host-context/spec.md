## ADDED Requirements

### Requirement: Scoped trusted host attachment
The runtime SHALL attach host workspace resources only for a host-authenticated, verified owner and workspace selecting an existing team and exact saved binding revision. Canonical working directory and server names SHALL match the private binding extension.

#### Scenario: Renderer or foreign scope cannot attach
- **WHEN** attachment lacks trusted host authentication or selects a foreign owner/workspace, directory or binding revision
- **THEN** the runtime rejects attachment before admitting a tool resource

### Requirement: Exact member policy and paired authority
The runtime SHALL propagate ordinary run MCP resources and paired host tool admission while retaining immutable member selection and current team authority. Attachment SHALL NOT grant tools absent from member or host policy, and secrets SHALL remain ephemeral.

#### Scenario: Readonly review
- **WHEN** a reviewer invokes a write selected by another member
- **THEN** effective member policy and host role checks deny the effect

### Requirement: Restart requires reattachment
The runtime SHALL reject required-context dispatch with TEAM_HOST_CONTEXT_REQUIRED before claiming a queued attempt after ephemeral context loss.

#### Scenario: Trusted recovery
- **WHEN** Boss reopens a persisted coding instance after runtime restart and requests recovery
- **THEN** Boss explicitly reattaches the canonical saved workspace and paired authority before ordinary recovery, retaining existing fence and uncertain-effect rules
