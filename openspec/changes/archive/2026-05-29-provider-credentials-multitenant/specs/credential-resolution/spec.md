## ADDED Requirements

### Requirement: Scoped Resolution Chain
The `CredentialResolver` SHALL resolve a provider API key by trying scopes in strict
priority order — `session → agent → user → system → env var` — returning the first
match and `None` only when all scopes miss.

#### Scenario: Highest-priority scope wins
- **WHEN** credentials exist for the same provider at both the session and user scopes for a request
- **THEN** the resolver MUST return the session-scoped key.

#### Scenario: Fall-through to a lower scope
- **WHEN** no session or agent credential exists but a user-scoped credential exists for the provider
- **THEN** the resolver MUST return the user-scoped key.

#### Scenario: All scopes miss
- **WHEN** no credential exists at session, agent, user, system, or env for the provider
- **THEN** the resolver MUST return `None`.

#### Scenario: Resolution is provider-specific
- **WHEN** a user has a credential for provider P but not provider Q, and a request targets Q
- **THEN** resolution for Q MUST NOT return the credential stored for P.

### Requirement: Single-Tenant Env Fallback Equivalence
The terminal `system`/`env var` step of the resolution chain SHALL delegate to the
existing process-global registry/config key, so that with no per-user credentials
stored the system behaves identically to the current single-tenant configuration.

#### Scenario: Zero-config single-tenant resolution
- **WHEN** no `ProviderService` is configured (or no per-user/session/agent credential exists) and a provider key is present via env/config (`UAR_LLM__*`, `LLM_*`, provider-specific env, or config file)
- **THEN** resolution MUST return that env/config key, matching today's `src/config.rs` precedence behavior.

#### Scenario: No regression when credentials subsystem is absent
- **WHEN** `AppState.provider_service` is `None`
- **THEN** every request MUST resolve its key exactly as it does today, with no added failure modes.

#### Scenario: Operator house-account fallback in multi-tenant mode
- **WHEN** the credentials subsystem is enabled but a given user has no stored credential for the requested provider
- **THEN** resolution MUST fall through to the env/config key (the operator's house account) rather than failing.

### Requirement: Per-Request Resolution from UserContext
The chat/completions route SHALL resolve the credential for each request from the
request's `UserContext` and target provider before invoking the LLM, without changing
the `Orchestrator` chat method signatures.

#### Scenario: Authenticated request uses the resolved user key
- **WHEN** an authenticated request (carrying `UserContext`) targets a provider for which the user has a stored credential
- **THEN** the route MUST resolve that key and configure the per-request driver with it, such that the upstream call is authenticated with the user's key.

#### Scenario: Orchestrator signature unchanged (resolve-then-construct)
- **WHEN** the route integrates credential resolution
- **THEN** it MUST resolve the credential and construct/configure the per-request driver with the result, and MUST NOT add credential or `UserContext` parameters to `Orchestrator::chat` / `chat_with_history`.

#### Scenario: Anonymous or unscoped request falls back
- **WHEN** a request has no per-user credential (anonymous, or user without a stored key)
- **THEN** the route MUST proceed using the env/config fallback key (single-tenant path) rather than rejecting the request.

#### Scenario: Per-request model override resolves against its provider
- **WHEN** a request overrides the model to a different provider than the default
- **THEN** resolution MUST be performed against the overridden provider, not the default provider.

### Requirement: No Plaintext Leakage Through Resolution
The resolution path SHALL decrypt credentials only in memory for immediate use and
SHALL NOT log, serialize, or otherwise expose plaintext keys.

#### Scenario: Resolved keys are never logged
- **WHEN** the resolver decrypts and returns a key for a request
- **THEN** the plaintext key MUST NOT appear in logs, traces, error messages, or response bodies.

#### Scenario: Decryption failure is surfaced without the secret
- **WHEN** a stored credential fails to decrypt during resolution
- **THEN** the resolver MUST surface an error identifying the provider/scope but MUST NOT include ciphertext or partial plaintext in the error.
