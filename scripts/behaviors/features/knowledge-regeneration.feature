@api @knowledge-regeneration
Feature: Retrieve knowledge after changing the embedding model
  Scenario: Regenerate a retained document in a different embedding space
    Given a disposable knowledge base indexed with the original embedding model
    When I change the active embedding model to a different vector dimension
    Then the old knowledge index refuses incompatible retrieval
    When I regenerate the knowledge index from its retained document
    Then retrieval returns the original private fact using the new embedding space
