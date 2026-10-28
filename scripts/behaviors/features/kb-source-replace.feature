@api @knowledge @replacement
Feature: Replacing a KB source removes the old fact
  Scenario: Delete the old document, index a replacement, and answer from it only
    Given a disposable knowledge base with one indexed note holding a private value
    And a disposable agent linked to that knowledge base with citations required
    When I delete the old note and index a replacement note with a different value
    Then retrieval returns only the replacement note
    And a fresh conversation answers with only the replacement value and its current citation
