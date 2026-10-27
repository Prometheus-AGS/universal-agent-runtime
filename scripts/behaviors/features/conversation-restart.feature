@api @conversation @restart
Feature: Conversation survives a UAR restart
  Scenario: Recall earlier facts from the durable conversation after restart
    Given a disposable agent and a fresh durable conversation
    When I give it fresh facts before the restart
    And the owned UAR restarts with its existing store
    And I ask about those facts in the same conversation without replaying them
    Then the restarted UAR answers from the persisted earlier conversation
