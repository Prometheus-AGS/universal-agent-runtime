@api @knowledge @restart
Feature: Saved mathematics knowledge remains usable after restart
  Scenario: Retrieve the saved math agent's research note after UAR restarts
    Given the saved math agent retains its indexed research knowledge base
    When UAR restarts without changing the agent or knowledge store
    And I ask the math agent about its retained research profile in a fresh conversation
    Then the answer uses the retained document with matching retrieval evidence
