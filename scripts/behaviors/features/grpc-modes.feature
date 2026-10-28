@api @grpc @a2a
Feature: Legacy and v1 A2A gRPC unary and streamed completions
  Scenario: Legacy gRPC MessageSend completes and continues the same task
    Given a disposable saved agent with a private instruction marker for gRPC
    When I send a fresh calculation and then a dependent follow-up through legacy gRPC MessageSend
    Then both legacy gRPC tasks complete with the agent's marker and the correct answers

  # The legacy stream is a single status snapshot by contract; the client resolves that task with TaskGet.
  Scenario: Legacy gRPC MessageStream snapshots resolve to completed tasks and continue the same task
    Given a disposable saved agent with a private instruction marker for gRPC
    When I send a fresh calculation and then a dependent follow-up through legacy gRPC MessageStream
    Then each legacy gRPC stream is one status snapshot whose task resolves through TaskGet to the agent's marker and the correct answer

  Scenario: A2A v1 gRPC SendMessage completes and continues in the same context
    Given a disposable saved agent with a private instruction marker for gRPC
    When I send a fresh calculation and then a dependent follow-up through A2A v1 gRPC SendMessage
    Then both v1 gRPC tasks complete with the agent's marker and the correct answers in one context

  Scenario: A2A v1 gRPC SendStreamingMessage streams to a completed task in the same context
    Given a disposable saved agent with a private instruction marker for gRPC
    When I send a fresh calculation and then a dependent follow-up through A2A v1 gRPC SendStreamingMessage
    Then both v1 gRPC streams progress to a completed task with the agent's marker and the correct answers in one context
