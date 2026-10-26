@api @agents @saved-agent-query
Feature: Converse with a saved specialist on the running UAR
  As a user
  I want to reuse the agent I already saved
  So that I can keep working across conversations without restarting UAR

  Scenario: Query the previously created mathematics specialist
    Given the saved mathematics specialist is available in the running UAR
    When I ask that specialist to add two numbers
    Then its final answer uses its saved identity and the correct sum
