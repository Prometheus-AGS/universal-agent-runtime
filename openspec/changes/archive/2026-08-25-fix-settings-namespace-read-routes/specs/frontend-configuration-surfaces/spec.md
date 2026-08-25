## ADDED Requirements

### Requirement: Settings namespace reads use canonical backend slugs
The settings frontend SHALL translate internal namespace keys to the backend's canonical URL slug before every namespace read. The translation MUST preserve the existing response decoding and non-success error propagation, and MUST NOT change save routes, payloads, persistence, provider configuration, or realtime state.

#### Scenario: Provider namespace is read
- **WHEN** the frontend reads the internal `provider` namespace
- **THEN** it requests `/api/uar/settings/providers`
- **AND** it does not request `/api/uar/settings/provider`

#### Scenario: Underscored namespace is read
- **WHEN** the frontend reads the internal `context_management` namespace
- **THEN** it requests `/api/uar/settings/context-management`
- **AND** it does not issue an underscored settings namespace request

#### Scenario: Canonical namespace is unchanged
- **WHEN** the frontend reads the internal `server` namespace
- **THEN** it requests `/api/uar/settings/server`

#### Scenario: Canonical settings read fails
- **WHEN** a canonical settings namespace response is non-successful
- **THEN** the existing status-based error propagation is preserved
- **AND** the UI does not report a successful load

#### Scenario: Installed runtime settings are inspected
- **WHEN** the production static bundle is served by the installed runtime on port 1906 and an operator opens Provider Overrides and Context Management
- **THEN** the configured provider records render and Context Management loads through its hyphenated route
- **AND** the browser emits no singular, underscored, or other settings namespace 404
