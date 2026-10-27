@api @linked-kb-delete
Feature: Continue using an agent after deleting its linked knowledge base
  Scenario: A linked agent answers without deleted knowledge in a fresh conversation
    Given a disposable agent linked to a knowledge base containing a private fact
    And the agent answers using that fact with a matching knowledge citation
    When I delete its knowledge base without editing the agent
    Then the same agent answers a fresh conversation without the deleted fact or citations
