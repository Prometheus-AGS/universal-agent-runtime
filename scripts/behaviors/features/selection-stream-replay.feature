@api @agui @orchestrator
Feature: Reconnecting or replaying an orchestrator selection stream does not dispatch again
  Scenario: Cursor reconnect and identical replay preserve the selection run, and a next turn selects afresh
    Given the saved mathematics specialist is in the live roster
    When the orchestrator routes a fresh calculation and I capture its selection stream and dispatches
    And I reconnect to that run from the selection event's cursor
    And I replay the identical admitted request
    Then both replays preserve the run and selection identity and no second dispatch occurred
    When I send a distinct next turn on the same thread
    Then the next turn performs a fresh selection with its own dispatch and a correct answer
