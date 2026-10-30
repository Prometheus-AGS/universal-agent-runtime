Feature: The Anthropic Messages endpoint runs the same agent turn as chat completions

  /v1/messages and /v1/chat/completions both run the selected agent; only the wire
  format differs (#396). The keyless stub model makes both answers deterministic.

  Scenario: A text answer and its usage match across both endpoints
    Given a dedicated UAR with the stub model
    When I ask the default agent through chat completions
    And I ask the default agent through Messages
    Then both endpoints return the same answer
    And both endpoints agree that the model reported no token usage

  Scenario: A caller tool call matches across both endpoints
    Given a dedicated UAR with the stub model
    When I ask the default agent to use my lookup_code tool through chat completions
    And I ask the default agent to use my lookup_code tool through Messages
    Then chat completions returns a lookup_code tool call
    And Messages stops with tool_use and a lookup_code block with the same input
