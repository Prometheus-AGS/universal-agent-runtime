@api @sdk @rust
Feature: Shipped Rust SDK unary and streamed completions
  Scenario: The Rust SDK's unary chat answers and continues a session with a saved agent
    Given a disposable saved agent with a private instruction marker for the Rust SDK
    When the shipped Rust SDK sends a fresh calculation and then a dependent follow-up with chat().complete
    Then both Rust SDK completions stop with the agent's marker and correct answers in one session

  Scenario: The Rust SDK's streamed chat answers and continues a session with a saved agent
    Given a disposable saved agent with a private instruction marker for the Rust SDK
    When the shipped Rust SDK sends a fresh calculation and then a dependent follow-up with chat().stream
    Then both Rust SDK streams deliver incremental text to one stop with correct answers in one session
