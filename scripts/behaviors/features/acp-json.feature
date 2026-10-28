@api @acp
Feature: Non-streamed ACP request returns the agent's answer
  Scenario Outline: A non-streaming ACP client gets the <agent> agent's answer
    Given an ACP session targets the <agent> agent
    When the client sends a fresh calculation with a non-streamed ACP runs/create
    Then an ACP response delivers the completed run's correct answer to the client

    Examples:
      | agent  |
      | default |
      | custom |
