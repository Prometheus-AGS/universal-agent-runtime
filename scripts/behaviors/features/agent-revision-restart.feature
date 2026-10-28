@api @agents @restart
Feature: Updated and deleted agent revisions survive restart
  Scenario: A changed agent revision stays effective across restart and a deleted agent stays gone
    Given a disposable agent whose instructions were changed after creation
    When UAR restarts with the existing agent store
    Then the agent's saved definition is the changed revision
    And a fresh prompt follows only the changed instructions
    When I delete the disposable agent
    And UAR restarts with the existing agent store again
    Then the deleted agent cannot be read or listed after cold startup
    And selecting the deleted agent dispatches no model call
