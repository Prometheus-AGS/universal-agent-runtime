@api @knowledge @restart
Feature: A deleted knowledge base stays deleted after cold startup
  Scenario: A linked agent cannot retrieve deleted knowledge after UAR restarts
    Given a disposable agent linked to a knowledge base containing a private fact
    And the agent answers using that fact and direct retrieval finds it
    When I delete its knowledge base without editing the agent
    And UAR restarts with the existing knowledge store
    Then the knowledge base, its document and its retrieval are absent after cold startup
    And the agent still references the deleted knowledge base unchanged
    And the agent answers a fresh conversation without the deleted fact or citations
