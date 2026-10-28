@api @memory
Feature: A memory opt-out survives restart and stops new capture
  Scenario: After opting out and a restart, a new private fact is never remembered
    Given durable memory with automatic capture is enabled for a disposable agent after an owned restart
    When I opt in, tell the agent a private fact, and ask about it in a fresh conversation
    Then the agent recalls the fact from memory
    When I opt out of memory capture and the runtime restarts
    Then my opt-out is still in effect
    When I tell the agent a new private fact and ask about it in a fresh conversation
    Then the new fact was never captured and the agent cannot recall it
