@api @agents @persistence
Feature: Save a specialist agent for later use
  As a user
  I want to save and prompt my own specialist agent
  So that it remains available for later conversations and orchestrator routing

  Scenario: Query a saved mathematics specialist after restarting UAR
    Given a dedicated UAR instance with durable storage and a configured real model
    When I create and save a mathematics specialist agent
    And I restart that UAR instance
    Then the saved agent definition is still available
    And querying the agent follows its saved instructions
