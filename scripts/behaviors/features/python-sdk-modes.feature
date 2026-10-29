@api @sdk @python
Feature: Shipped Python SDK unary and streamed completions
  Scenario: The Python SDK's unary chat answers and continues a session with a saved agent
    Given a disposable saved agent with a private instruction marker for the Python SDK
    When the shipped Python SDK sends a fresh calculation and then a dependent follow-up with chat
    Then both SDK completions stop with the agent's marker and correct answers in one session

  Scenario: The Python SDK's streamed chat answers and continues a session with a saved agent
    Given a disposable saved agent with a private instruction marker for the Python SDK
    When the shipped Python SDK sends a fresh calculation and then a dependent follow-up with stream_chat
    Then both SDK streams deliver incremental text to one stop with correct answers in one session
