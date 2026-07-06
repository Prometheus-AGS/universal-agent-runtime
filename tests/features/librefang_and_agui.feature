@api
Feature: LibreFang integration surface and AG-UI event vocabulary
  As an external orchestrator (e.g. LibreFang/bossfang)
  I need UAR's OpenAI-compatible endpoint and AG-UI event stream to behave
  exactly as documented in docs/librefang-integration.md
  So that I can integrate with zero UAR-side code changes (CH-18, D-C scope)

  Background:
    Given a running UAR server with a stub LLM

  Scenario: Zero-code bossfang seam via /v1/chat/completions
    Given the stub LLM responds to "hello from a bossfang-shaped client" with the content "hi, this is UAR"
    When I send a bare OpenAI-shaped chat completion request with message "hello from a bossfang-shaped client" to "/v1/chat/completions"
    Then the response status should be successful
    And the response body should be an OpenAI chat.completion with content "hi, this is UAR"

  Scenario: agui_spec mode emits the official AG-UI event vocabulary
    Given the stub LLM responds to "hello agui spec mode" with the content "hi there"
    When I send a streaming chat completion request with stream_mode "agui_spec" and message "hello agui spec mode"
    Then the response should contain the event "RUN_STARTED"
    And the response should contain the event "TEXT_MESSAGE_CONTENT"
    And the response should contain the event "RUN_FINISHED"
    And the response should not contain the legacy event "agui.stream.start"
    And the response should not contain the legacy event "agui.message.delta"

  Scenario: agui_spec mode maps the tool-call lifecycle to the official vocabulary
    Given the stub LLM responds to "echo this via agui spec" with a call to tool "native_echo" then the content "echoed via the tool loop"
    When I send a streaming chat completion request with stream_mode "agui_spec" and message "echo this via agui spec"
    Then the response should contain the event "TOOL_CALL_END"
    And the response should contain the event "TOOL_CALL_RESULT"
    And the response should not contain the legacy event "agui.tool_call."

  Scenario: legacy agui mode is unaffected by the new agui_spec mode
    Given the stub LLM responds to "hello legacy agui mode" with the content "hi there"
    When I send a streaming chat completion request with stream_mode "agui" and message "hello legacy agui mode"
    Then the response should contain the legacy event "agui.stream.start"
    And the response should not contain the event "RUN_STARTED"
