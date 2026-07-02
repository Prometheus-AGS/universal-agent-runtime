## ADDED Requirements

### Requirement: Interchangeable live/recorded backend selection
The system SHALL provide a `live` integration test tier where every test
case runs against a resolved `base_url`, selected by
`UAR_LIVE_INTEGRATION_BACKEND` (default `recorded`): `live` resolves to the
real local OpenAI-compatible proxy; `recorded` resolves to an in-process
stub LLM server serving canned chat-completion responses (non-streaming and
SSE streaming, plain content and tool-call fixtures) keyed by a request
fingerprint. Both resolve through the same code path — there is no separate
implementation per backend to keep in sync.

#### Scenario: Default backend is recorded
- **WHEN** the live integration tier resolves a backend with no
  `UAR_LIVE_INTEGRATION_BACKEND` environment variable set
- **THEN** it starts the in-process stub LLM server and resolves to its
  base URL

#### Scenario: Explicit live backend targets the local proxy
- **WHEN** `UAR_LIVE_INTEGRATION_BACKEND=live` is set
- **THEN** the tier resolves to `http://127.0.0.1:8181/v1` with model
  `openai/gpt-5.4-mini`, and does not start an in-process server

#### Scenario: Recorded backend serves configured fixtures
- **WHEN** a test case registers a fixture keyed by model, last user
  message, and tool-schema presence, then sends a matching
  `/v1/chat/completions` request against the resolved recorded base URL
- **THEN** the stub server returns the fixture's response (streaming or
  non-streaming, content or tool-call, per the request's `stream` flag)

#### Scenario: Missing fixture fails clearly
- **WHEN** a `/v1/chat/completions` request against the stub server does not
  match any registered fixture
- **THEN** the stub server returns an HTTP 404 whose body names the
  unmatched model, last user message, and tool-schema presence

### Requirement: Proxy health check with actionable remediation
The system SHALL provide `scripts/live-integration.sh`, which performs a
health check against the configured live proxy before running any live-backend
test case. On health-check failure, the script SHALL print the specific
remediation steps (re-authenticate the Codex-backed proxy; restart the
`ai.prometheus.openai-proxy` launchd service) and exit non-zero without
running any test case.

#### Scenario: Healthy proxy allows the live run to proceed
- **WHEN** `scripts/live-integration.sh` is invoked and the configured proxy
  responds successfully to the health check
- **THEN** the script sets `UAR_LIVE_INTEGRATION_BACKEND=live` and runs the
  live-backend test cases

#### Scenario: Unhealthy proxy fails fast with remediation
- **WHEN** `scripts/live-integration.sh` is invoked and the configured proxy
  does not respond successfully to the health check, and
  `--allow-recorded-fallback` is not set
- **THEN** the script exits non-zero, prints the Codex re-login step and the
  `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy` remediation
  command, and does not execute any test case

#### Scenario: CI falls back to the recorded backend without a proxy
- **WHEN** `scripts/live-integration.sh` runs with `--allow-recorded-fallback`
  set and the local proxy is unreachable
- **THEN** the script skips the failure path and runs the test cases against
  the `recorded` backend instead of failing
