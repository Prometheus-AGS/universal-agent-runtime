## ADDED Requirements

### Requirement: Messages requests run the selected agent
`POST /v1/messages` SHALL run a registered agent, chosen from the request `agent_id`, then the conversation policy, then `default-agent`. It SHALL refuse `uar:agent:*` targets with 400.

#### Scenario: Text parity with chat completions
- **WHEN** the same prompt is sent to the default agent through `/v1/chat/completions` and `/v1/messages`
- **THEN** both return the same answer text and the same output token count.

#### Scenario: Caller tool parity
- **WHEN** the same prompt and caller tool are sent through both endpoints and the model calls the tool
- **THEN** chat completions returns that tool call, and Messages stops with `tool_use` and a block with the same name and input.
