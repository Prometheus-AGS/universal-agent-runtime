## ADDED Requirements

### Requirement: Authenticated interoperable A2A
The runtime SHALL serve official A2A 1.0 JSON-RPC and gRPC over the durable task service and advertise only implemented interfaces while documenting preserved legacy surfaces. Operator-required skill roles SHALL be enforced before reservation and during dynamic activation and authenticated continuation.

#### Scenario: Protected explicit skill is unauthorized
- **WHEN** the verified caller lacks any operator-required role for an explicitly preferred protected skill
- **THEN** the runtime returns HTTP 403 or gRPC PERMISSION_DENIED and creates no task

### Requirement: Agent-bound MCP authority
The runtime SHALL enforce optional allowed_agent_ids before initialization, discovery and invocation. Absent preserves compatibility; empty denies all. Restricted destinations require explicit trusted-host grants bound to persisted agent identity, revision and current policy. Inline artifacts SHALL NOT impersonate persisted agents.

#### Scenario: Same-owner other agent
- **WHEN** agent B or an inline artifact uses credentials for a destination restricted to persisted agent A
- **THEN** no peer contact occurs, including after restart and continuation

### Requirement: Durable remote manifest import
The runtime SHALL accept a single skill_markdown through POST /api/uar/skills/import-content and return exact content and SHA-256 digest through normal GET. Success SHALL follow checked durable persistence and collisions SHALL be rejected.

#### Scenario: Storage fails
- **WHEN** durable creation fails
- **THEN** no successful response or in-memory skill is published

### Requirement: Verified release configuration and interactions
Shipped configurations SHALL require authentication and SHALL NOT select an implicit model. Default startup SHALL remain loopback, offline and model-free; explicit deployments MAY use configured network listeners and remote databases. Required approvals SHALL prevent invocation until accepted, and elicitation SHALL support both sharing modes with schema-valid AG-UI interruption/resume.

#### Scenario: Published image acceptance
- **WHEN** v1.1.0 is published
- **THEN** its immutable digest is pulled and passes local production-path conformance before OG-4 success is reported
