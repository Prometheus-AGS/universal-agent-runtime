@api @agents
Feature: Built-in agents remain available
  Scenario: The default and orchestrator agents cannot be deleted
    Given the default and orchestrator agent definitions are available
    When I attempt to delete each built-in agent
    Then both deletions are refused because the agents are built in
    And both agent definitions remain unchanged
