## Purpose

Define durable prompt-caching controls, precedence, isolation, provider behavior, and observable evidence across supported runtime request paths.

## ADDED Requirements

### Requirement: Prompt caching has an authoritative global default
The runtime SHALL register a `prompt_caching.enabled` setting whose seeded value is `false`. Admin-authorized clients SHALL be able to read and update that setting through `GET` and `PUT /api/uar/settings/prompt-caching`, while unauthorized clients MUST NOT read or change it.

#### Scenario: A new installation reads prompt caching settings
- **WHEN** no operator has changed the prompt-caching setting
- **THEN** the settings API returns `enabled: false`

#### Scenario: An administrator updates the global default
- **WHEN** an admin-authorized client writes a valid prompt-caching setting
- **THEN** a subsequent read returns the saved value

#### Scenario: A non-administrator uses the global settings endpoint
- **WHEN** a client without admin authorization reads or updates prompt-caching settings
- **THEN** the request is rejected without exposing or changing the value

### Requirement: Effective prompt caching uses deterministic precedence
For a session-bound inference request, the runtime SHALL resolve prompt caching in this order: explicit request override, persisted session override, verified JWT user override, and global default. The session effective-state endpoint SHALL return `enabled`, `source`, `session_override`, `user_override`, and `global_default` without exposing another owner's session state.

#### Scenario: A request override is present
- **WHEN** a compatible inference request supplies `prompt_caching_enabled`
- **THEN** that value controls only the current request and its source is `request`

#### Scenario: A session override is present without a request override
- **WHEN** a session has a persisted prompt-caching override
- **THEN** that value applies to every subsequent policy-bearing turn in the session and its source is `session`

#### Scenario: A user override is present without a higher-priority override
- **WHEN** a verified principal has a prompt-caching preference and the request and session inherit
- **THEN** the user preference is effective and its source is `user`

#### Scenario: Every override inherits
- **WHEN** no request, session, or user override is set
- **THEN** the global setting is effective and its source is `global`

#### Scenario: Another owner requests effective state
- **WHEN** a principal requests prompt-caching state for a session it does not own
- **THEN** the response reveals no session configuration or effective values

### Requirement: Session prompt-caching overrides persist as tri-state policy
Session configuration SHALL store prompt caching as a nullable boolean, where missing or `null` means inherit and boolean values mean explicit On or Off. Legacy session policies without the field MUST deserialize as inherit.

#### Scenario: A session override is saved
- **WHEN** an owner saves On or Off in Session Configuration
- **THEN** reopening the session returns the same explicit value and subsequent turns use it unless a request override is supplied

#### Scenario: A session override is cleared
- **WHEN** an owner saves Inherit
- **THEN** the persisted session override is null and lower-precedence policy controls subsequent turns

#### Scenario: A legacy session is loaded
- **WHEN** stored configuration has no prompt-caching field
- **THEN** it loads successfully with an inherited prompt-caching policy

### Requirement: Missing owner-scoped agent configuration is indistinguishable
The owner-scoped session agent-configuration endpoint SHALL return an empty `204 No Content` response when configuration is absent. A cross-owner request MUST return the identical empty response, and a successful first save SHALL create configuration that a subsequent owner read returns.

#### Scenario: Configuration has not been created
- **WHEN** the session owner reads agent configuration before the first save
- **THEN** the endpoint responds with status 204 and an empty body

#### Scenario: A different owner probes configuration
- **WHEN** a principal reads another owner's session agent configuration
- **THEN** the endpoint responds with the same status and empty body as absent configuration

#### Scenario: The owner performs the first save
- **WHEN** the owner saves complete session configuration after an empty read
- **THEN** the write succeeds and a subsequent owner read returns the saved configuration

### Requirement: User prompt-caching preferences preserve four update states
The JWT-only user-settings update contract SHALL treat an omitted prompt-caching field as preserve, JSON `null` as clear, `true` as enable, and `false` as disable. Verified tenant identity plus JWT subject SHALL identify a principal when tenant identity exists; otherwise the verified subject SHALL identify the principal. Distinct principals MUST NOT observe or overwrite one another's values.

#### Scenario: The field is omitted
- **WHEN** an authenticated user updates other settings without a prompt-caching field
- **THEN** the stored prompt-caching preference is unchanged

#### Scenario: The field is null
- **WHEN** an authenticated user writes `prompt_caching_enabled: null`
- **THEN** the user preference is cleared and precedence falls through to the global default

#### Scenario: The field is boolean
- **WHEN** an authenticated user writes true or false
- **THEN** that exact value is stored and returned

#### Scenario: Two principals use user settings
- **WHEN** two verified principals read and update preferences
- **THEN** each principal sees only its own record

### Requirement: User preferences use configured durable persistence
User prompt-caching preferences SHALL write through to the configured Memory, Postgres, or Surreal persistence backend and SHALL survive runtime reconstruction when that backend is available. When durable persistence is unavailable, the runtime MAY fall back to process-lifetime storage but MUST expose and document that limitation.

#### Scenario: A configured durable backend reloads
- **WHEN** a user preference is saved and the runtime store is reconstructed against the same backend
- **THEN** the saved value is restored

#### Scenario: Durable persistence is unavailable
- **WHEN** no configured backend can persist user settings
- **THEN** the API remains functional with process-lifetime storage and operators can identify the fallback limitation

### Requirement: Provider dispatch applies effective caching without changing OpenAI
Every production policy-bearing LLM request path, including initial chat, tool-loop iterations, compatibility requests, and failover, SHALL use one effective cache-strategy boundary. Anthropic requests SHALL use the native driver while its feature gate is enabled and SHALL include ephemeral cache controls only when effective caching is On. Feature-gate fallback SHALL remain available through the compatible driver. OpenAI dispatch and request bodies MUST be identical for On and Off values because OpenAI manages eligible caching automatically.

#### Scenario: Anthropic caching is on
- **WHEN** a policy-bearing Anthropic request resolves prompt caching to On
- **THEN** the upstream request contains the supported ephemeral cache controls across initial, iterative, compatibility, and failover paths

#### Scenario: Anthropic caching is off
- **WHEN** a policy-bearing Anthropic request resolves prompt caching to Off
- **THEN** the upstream request contains no cache-control blocks across those paths

#### Scenario: Native Anthropic support is disabled
- **WHEN** the Anthropic native-driver feature is not compiled
- **THEN** Anthropic routing remains available through the compatible fallback without claiming native cache-control behavior

#### Scenario: OpenAI receives either effective value
- **WHEN** otherwise identical OpenAI requests resolve prompt caching to On and Off
- **THEN** their provider dispatch and upstream request bodies remain identical

### Requirement: Prompt caching exposes provider usage evidence
The runtime SHALL map provider-reported cache creation and cache-read usage into deterministic prompt-caching metrics without fabricating cache activity when a provider reports none. Internal LLM calls that do not inherit user policy MUST be documented.

#### Scenario: A provider reports cache usage
- **WHEN** a mocked provider response includes cache-write or cache-read token usage
- **THEN** the corresponding runtime metrics record the exact reported values

#### Scenario: A provider reports no cache usage
- **WHEN** a provider response omits cache usage
- **THEN** the runtime does not infer or report a cache hit
