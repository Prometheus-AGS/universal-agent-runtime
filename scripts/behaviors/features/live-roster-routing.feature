@api @agents @orchestrator @routing
Feature: Intent routing follows the live eligible roster
  Scenario: Route to a new specialist only while it is eligible
    Given the saved mathematics and writing specialists are in the live roster
    When I register a disposable houseplant care specialist
    Then the live roster includes the houseplant specialist
    When I ask the orchestrator to diagnose a houseplant problem
    Then intent routing selects the houseplant specialist before its answer
    And the houseplant specialist answers with its private clinic signature
    When I delete the houseplant specialist
    Then the live roster no longer includes the houseplant specialist
    When I ask the orchestrator a fresh houseplant question
    Then routing uses only a currently eligible agent and never the deleted specialist
    When I ask the orchestrator a request that matches no specialist
    Then that request is also routed only within the current eligible roster
