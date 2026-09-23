## Purpose

Lets a host that owns provider credentials hand one provider — kind, endpoint and key — to one run, so UAR runs the agent loop against that endpoint with that key without storing either, sharing them with another session, or keeping them after the run ends.

## ADDED Requirements

### Requirement: A run accepts host-supplied provider credentials
`POST /api/uar/runs` SHALL accept an optional `run_credentials` list whose entries carry a `provider_id`, a `provider_kind`, a `base_url`, an `api_key` and an optional `default_model`. Each `provider_id` SHALL match `^[a-z0-9][a-z0-9._-]{0,62}$` and SHALL appear at most once; it need not name a provider in UAR's registry. An empty `api_key`, an invalid or duplicate `provider_id`, an invalid `base_url`, or more entries than the configured maximum SHALL be rejected with HTTP 422 and error code `run_credential_invalid`, and no run SHALL start. The error body SHALL NOT contain any submitted key or base URL.

#### Scenario: Valid run credential
- **WHEN** a host creates a run with one run credential of kind `openai_compatible` whose provider id the agent's policy names
- **THEN** the run starts and every model request for that provider goes to the credential's base URL and carries the credential's key

#### Scenario: Provider unknown to the registry
- **WHEN** a run credential names a provider id that is not in UAR's provider registry
- **THEN** the run starts and uses the credential's kind, base URL and key for that provider id

### Requirement: A run credential's base URL is validated before the run starts
A run credential's `base_url` SHALL be an absolute URL with a host. Its scheme SHALL be `https`, or `http` only when the host is a loopback address or `localhost`. It SHALL NOT carry a username or password. When local-only mode is on, a non-loopback host SHALL be rejected. A URL that fails any rule SHALL be rejected with HTTP 422 and code `run_credential_invalid` before any connection is attempted, and the response SHALL NOT contain the URL.

#### Scenario: Plain http to a remote host
- **WHEN** a run credential's `base_url` is `http://models.example.com/v1`
- **THEN** the response is 422 with `run_credential_invalid`, no request is sent to that host, and the body does not contain the URL

#### Scenario: Userinfo in the URL
- **WHEN** a run credential's `base_url` is `https://user:pass@gateway.example.com/v1`
- **THEN** the response is 422 with `run_credential_invalid` and no run starts

#### Scenario: Plain http to a local model server
- **WHEN** a run credential's `base_url` is `http://127.0.0.1:11434/v1` with kind `openai_compatible`
- **THEN** the run starts and its model requests go to that address

### Requirement: The provider kind selects the driver, and unsupported kinds fail clearly
UAR SHALL build a run credential's model client from its `provider_kind`, not from its `provider_id`. Kind `openai_compatible` SHALL use the OpenAI-compatible chat protocol at the base URL. Kind `anthropic` SHALL use the Anthropic Messages protocol at the base URL. The model id SHALL be the routed model for that provider, else the credential's `default_model`; with neither, the run SHALL end with `run_credential_provider_unavailable`. Any other kind SHALL be rejected with HTTP 422 and code `run_credential_provider_kind_unsupported`, whose message names the rejected kind and the supported kinds and contains no key or URL, and no run SHALL start. When `provider_id` equals a provider in UAR's registry, the run SHALL NOT use that registry entry's key, base URL, default model or model list.

#### Scenario: Custom endpoint receives the requests
- **WHEN** a run credential of kind `openai_compatible` points at a mock server's base URL and the registry holds an entry with the same provider id and a different base URL
- **THEN** every model request of the run reaches the mock server and none reaches the registry entry's base URL

#### Scenario: Anthropic kind under a custom provider id
- **WHEN** a run credential of kind `anthropic` uses provider id `claude-work` and a mock base URL
- **THEN** the mock receives Anthropic Messages requests, not OpenAI-compatible chat requests

#### Scenario: Unsupported kind
- **WHEN** a run credential names provider kind `bedrock`
- **THEN** the response is 422 with `run_credential_provider_kind_unsupported` naming `bedrock` and the supported kinds, and no run id is issued

### Requirement: A run credential wins over every other key for its provider
For a provider named by a run credential, the run SHALL use the run credential's key and base URL and SHALL NOT use a session, agent, user or system credential, a provider-registry key or base URL, a configured key or base URL, or an environment-variable key, whatever the resolution order would otherwise select. The run credential SHALL be applied after model routing has chosen the provider.

#### Scenario: Stored credentials exist for the same provider
- **WHEN** a user-scoped stored credential, a registry key and an environment key exist for provider P and a run carries a run credential for P
- **THEN** every request the run sends to P carries the run credential and none carries another key

### Requirement: A run with run credentials only reaches credentialed providers
When a run carries at least one run credential, model routing, failover, skill-preferred models, and child runs SHALL select only providers that have a run credential. If the selected or required provider has none, the run SHALL end with a run error whose code is `run_credential_provider_unavailable`, and SHALL NOT send any request to that provider with any other key. No failure received under a run credential SHALL count toward the shared provider health cooldown.

#### Scenario: Failover target has no run credential
- **WHEN** the primary provider fails and the configured fallback belongs to a provider with no run credential
- **THEN** no request reaches the fallback provider and the run ends with `run_credential_provider_unavailable`

#### Scenario: One session's bad key
- **WHEN** runs in one session repeatedly receive 401 from provider P under a wrong run credential
- **THEN** a concurrent run in another session with a valid run credential for P is still routed to P

### Requirement: Child runs inherit the parent's credentials in memory only
A child run started by a run with run credentials SHALL use the parent's already-constructed model clients and SHALL NOT resolve credentials again. A run credential SHALL NOT be sent to a remote agent peer, written into a delegated message, or exposed to a child through any serializable value.

#### Scenario: Local child run
- **WHEN** a credentialed run spawns a local child agent
- **THEN** the child's provider requests carry the parent's run credential and no child record, message or event contains the key

#### Scenario: Remote peer delegation
- **WHEN** a credentialed run delegates to a remote UAR peer
- **THEN** the outbound request to the peer contains no run credential

### Requirement: Resume and continuation fail closed without re-attached credentials
A run created with run credentials SHALL record which provider ids it was given, never the keys. A resume (`POST /api/uar/runs/{id}/resume`, `POST /api/uar/runs/{id}/resume/{checkpoint_id}`) or A2UI continuation (`POST /api/uar/runs/{id}/a2ui/actions`) whose source run recorded run credentials SHALL carry `run_credentials` covering the same provider ids. Otherwise the request SHALL be rejected with HTTP 422 and code `run_credential_required`, and no run SHALL start and no provider request SHALL be sent. UAR SHALL NOT keep a run credential after its run and all descendants reach a terminal state, so a resumed or continued run never draws on a retained copy.

#### Scenario: Resume without credentials
- **WHEN** a host resumes a credentialed run and omits `run_credentials`
- **THEN** the response is 422 with `run_credential_required`, no run id is issued, and the mock provider receives no request

#### Scenario: A2UI action without credentials
- **WHEN** a host posts an A2UI action for a surface of a credentialed run and omits `run_credentials`
- **THEN** the response is 422 with `run_credential_required` and no continuation run starts

#### Scenario: Resume with re-attached credentials
- **WHEN** the host resumes the same run with a run credential for the recorded provider
- **THEN** the resumed run starts and its provider requests carry the re-attached key

#### Scenario: Source run unknown after a sidecar restart
- **WHEN** a host resumes or continues a run id the restarted process does not hold
- **THEN** the response is 404 and no run starts

### Requirement: Run credentials never leave process memory
A run credential's key and base URL SHALL NOT appear in any serialized request or response, run record, event, SSE frame, checkpoint, conversation history, database row, file, log line, trace span or attribute, metric label, or error message, including provider error bodies that echo the key and transport errors that name the URL. Both SHALL be dropped when the run and all descendants reach a terminal state.

#### Scenario: Byte search after credentialed runs
- **WHEN** runs with distinct run credentials complete, including a child run, a resumed run, a checkpoint write and reload, and a provider error whose response body echoes the key
- **THEN** a byte search of UAR's data directories, captured logs and traces, every SSE stream, and every run and checkpoint API response finds no key in raw or base64 form

#### Scenario: Base URL absent from logs and storage
- **WHEN** runs use a run credential whose base URL carries a unique marker in its path, including a run whose endpoint refuses the connection
- **THEN** a byte search of UAR's data directories, captured logs and traces at TRACE level, every SSE stream, and every run and checkpoint API response finds no occurrence of that marker

#### Scenario: Provider error echoes the key
- **WHEN** a provider responds with an error body that contains the request's key
- **THEN** the run error message, logs and traces contain the error without the key
