Feature: Chat with a tool call, invocation and result surfaced in the transcript
  As a user whose message triggers a tool call
  I want to see both the tool invocation and its result in the transcript
  So that I can trust the agent actually used a tool, not just described using one

  Scenario: Tool call invoked and result surfaced
    Given a fresh conversation with an agent that has tools enabled
    When I send the message "Echo this via the bdd tool-call scenario."
    Then a tool-call block for "native_echo" is shown in the transcript
    And the tool-call result contains "echoed via the tool loop"
