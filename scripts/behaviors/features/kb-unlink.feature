@api @kb-unlink
Feature: Unlink an agent from knowledge without deleting that knowledge
  Scenario: Unlinking stops agent retrieval while the KB remains available
    Given a disposable agent linked to a retained knowledge base containing a private fact
    And the linked agent uses the private fact with a matching knowledge citation
    When I remove only that knowledge base link from the agent
    Then the unlinked agent answers without the fact while the knowledge base remains searchable
