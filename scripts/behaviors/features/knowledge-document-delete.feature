@api @knowledge-document-delete
Feature: Remove deleted documents from knowledge retrieval
  Scenario: Delete one document without losing another document in the same KB
    Given a disposable knowledge base with two indexed laboratory notes
    And the same query retrieves both original notes
    When I delete only the first laboratory note
    Then retrieval excludes the deleted note and still returns the retained note
