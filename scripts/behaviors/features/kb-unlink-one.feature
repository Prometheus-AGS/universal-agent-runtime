@api @kb-unlink-one
Feature: Restrict agent knowledge after removing one of two KB links
  Scenario: Removing cobalt leaves only amber available to the agent
    Given a disposable agent with two indexed knowledge bases ready to link
    And both knowledge base links are persisted on that agent
    When I unlink only the cobalt knowledge base while leaving knowledge enabled
    And I ask for both inventories in a fresh conversation
    Then only the remaining knowledge base contributes facts and citations while both KBs stay searchable
