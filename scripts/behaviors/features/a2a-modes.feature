@api @a2a
Feature: Legacy A2A JSON and A2A v1 streamed tasks
  Scenario: Legacy A2A message/send completes and continues in the same context
    Given a disposable saved agent with a private instruction marker for A2A
    When I send a fresh calculation and then a dependent follow-up through legacy A2A message/send
    Then both legacy tasks complete with the agent's marker and the correct answers in one context

  Scenario: A2A v1 SendStreamingMessage streams task updates and continues in the same context
    Given a disposable saved agent with a private instruction marker for A2A
    When I send a fresh calculation and then a dependent follow-up through A2A v1 SendStreamingMessage
    Then both streams progress to a completed task with the agent's marker and the correct answers in one context
