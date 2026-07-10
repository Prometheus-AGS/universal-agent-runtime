@api
Feature: Core chat completion behaviors
  As a client of UAR's chat surface
  I need the OpenAI-compatible completion endpoint and the streaming surface
  to behave correctly across single-turn, multi-turn, tool-using, streaming,
  and malformed requests
  So that regressions in the core chat path are caught by outside-in scenarios.

  Background:
    Given a running UAR server with a stub LLM

  Scenario: Single-turn non-streaming completion returns assistant content
    Given the stub LLM responds to "what is the capital of france" with the content "Paris"
    When I send a bare OpenAI-shaped chat completion request with message "what is the capital of france" to "/v1/chat/completions"
    Then the response status should be successful
    And the response body should be an OpenAI chat.completion with content "Paris"

  Scenario: Multi-turn conversation answers the latest user turn
    Given the stub LLM responds to "and its population" with the content "About 2.1 million"
    When I send a chat completion request continuing the conversation: user "what is the capital of france", assistant "Paris", then user "and its population"
    Then the response status should be successful
    And the response body should be an OpenAI chat.completion with content "About 2.1 million"

  Scenario: Tool-call round trip produces final assistant content
    Given the stub LLM responds to "echo hello via a tool" with a call to tool "native_echo" then the content "echoed: hello"
    When I send a bare OpenAI-shaped chat completion request with message "echo hello via a tool" to "/v1/chat/completions"
    Then the response status should be successful
    And the response body should be an OpenAI chat.completion with content "echoed: hello"

  Scenario: Streaming request emits content and a terminal completion event
    Given the stub LLM responds to "stream me a greeting" with the content "hello there"
    When I send a streaming chat completion request with stream_mode "agui" and message "stream me a greeting"
    Then the response status should be successful
    And the response should contain the event "hello there"
    And the response should contain the legacy event "agui.done"

  Scenario: Malformed request without messages is a client error
    Given the stub LLM responds to "ignored" with the content "ignored"
    When I send a chat completion request with no "messages" field
    Then the response status should be a client error
