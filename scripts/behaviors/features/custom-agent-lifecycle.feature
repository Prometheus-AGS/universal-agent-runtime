@api @agents
Feature: Edit and remove a disposable custom agent
  Scenario: Updated instructions take effect before the custom agent is deleted
    Given a new disposable agent with an original instruction rule
    When I prompt it using the original rule
    Then its answer follows its original instructions
    When I save different instructions on that agent
    And I prompt it again in a fresh conversation
    Then its answer follows only the updated instructions
    When I delete the disposable agent
    Then its saved definition is no longer available
