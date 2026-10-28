@api @sdk @typescript
Feature: Shipped TypeScript SDK unary and streamed completions
  Scenario: The TypeScript SDK's unary chat answers and continues a session with a saved agent
    Given a disposable saved agent with a private instruction marker for the TypeScript SDK
    When the shipped TypeScript SDK sends a fresh calculation and then a dependent follow-up with chat.complete
    Then both TypeScript SDK completions stop with the agent's marker and correct answers in one session

  Scenario: The TypeScript SDK's streamed chat answers and continues a session with a saved agent
    Given a disposable saved agent with a private instruction marker for the TypeScript SDK
    When the shipped TypeScript SDK sends a fresh calculation and then a dependent follow-up with chat.stream
    Then both TypeScript SDK streams deliver incremental text to one stop with correct answers in one session
