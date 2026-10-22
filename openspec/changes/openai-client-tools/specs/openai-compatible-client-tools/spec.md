## ADDED Requirements

### Requirement: Complete caller-tool wire support
OpenAI, AG-UI, Anthropic Messages, A2A and repository ACP SHALL advertise, return and continue caller-executed tools through their native protocol representations and the shared trusted lifecycle.

#### Scenario: OpenAI streaming or JSON
- **WHEN** a request supplies function tools and full conversation history
- **THEN** definitions and explicit choice/parallel controls reach the selected provider or return an explicit unsupported capability error
- **AND** caller calls return as tool_calls with matching IDs and finish_reason tool_calls in the selected response form

#### Scenario: Native alternate interface
- **WHEN** the request uses AG-UI, Anthropic, A2A or ACP
- **THEN** native tool declarations/results and pending-task or terminal representations preserve the same ownership, claim and replay semantics

#### Scenario: Failure is not success
- **WHEN** budget, provider, cancellation, persistence or observation fails
- **THEN** the adapter reports a failure without a successful terminal marker

#### Scenario: Frozen release
- **WHEN** the new capability is released
- **THEN** it uses a separately authorized1.2.1 artifact and neither1.2.0 tag nor pinned index is changed
