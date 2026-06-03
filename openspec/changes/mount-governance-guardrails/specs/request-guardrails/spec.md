## ADDED Requirements

### Requirement: Cedar governance layer is mounted and enforces the loaded policy set

The server SHALL mount the Cedar `governance_layer` HTTP middleware so that
requests carrying an agent identity are authorized against the loaded policy
set. With the default permit-all policy this SHALL preserve existing behavior;
restrictive policies SHALL take effect at the HTTP layer once present.

#### Scenario: Permit-all default preserves behavior
- **WHEN** the default permit-all policy is loaded and any request is made
- **THEN** the governance layer allows the request and behavior is unchanged

#### Scenario: Restrictive policy denies
- **WHEN** a restrictive policy forbids an agent's action and a request carries that agent identity for that action
- **THEN** the governance layer returns `403` with a `GOVERNANCE_DENIED` code and the request does not proceed

#### Scenario: Anonymous requests pass through
- **WHEN** a request carries no agent identity
- **THEN** the governance layer does not deny it (authorization is skipped)

### Requirement: Chat input is screened for injection and PII

The server SHALL screen the user input on the chat path, before the LLM call,
with local heuristics for prompt-injection/jailbreak patterns and obvious
secrets/PII. Screening SHALL be local (no LLM or network call).

#### Scenario: Injection pattern detected
- **WHEN** input screening is enabled and the user input contains a known injection/jailbreak pattern
- **THEN** the input is flagged with the matched category before the LLM call

#### Scenario: Secret/PII pattern detected
- **WHEN** the user input contains an obvious secret or PII-shaped string (e.g. an API key or card/SSN pattern)
- **THEN** the input is flagged with the matched category

#### Scenario: Clean input is not flagged
- **WHEN** the user input matches no injection or PII heuristic
- **THEN** no guardrail flag is raised and the request proceeds normally

#### Scenario: Disabled screening is a no-op
- **WHEN** input screening is disabled
- **THEN** no screening runs and behavior is unchanged

### Requirement: Flagging surfaces an event and metric without leaking content

When input is flagged, the server SHALL record a guardrail metric, log a
warning, and emit a `GuardrailFlagged` event carrying the matched category and a
short reason. The event SHALL NOT include the full user input or the matched
secret value.

#### Scenario: Flag emits event and metric
- **WHEN** input is flagged by a guardrail heuristic
- **THEN** a `GuardrailFlagged` event is emitted (mapped to `agui.guardrail` on the SSE stream) and a guardrail counter is incremented, without the raw input or secret value

### Requirement: Blocking is opt-in; detection is non-blocking by default

Screening SHALL be detect-only by default (flag without blocking). When blocking
is enabled by configuration, a flagged input SHALL be rejected before the LLM
call with a clear error instead of being sent to the model.

#### Scenario: Detect-only default
- **WHEN** input is flagged and blocking is not enabled
- **THEN** the request still proceeds to the LLM (the flag is informational)

#### Scenario: Blocking enabled
- **WHEN** input is flagged and blocking is enabled
- **THEN** the request is rejected before the LLM call with a guardrail error and no run is started

### Requirement: Governance action vocabulary includes output validation

The governance action constants SHALL include `call_llm` and `validate_output`
so policies can target LLM invocation and response validation.

#### Scenario: Actions available to policies
- **WHEN** a Cedar policy references the `call_llm` or `validate_output` action
- **THEN** the action constant exists in the governance action model
