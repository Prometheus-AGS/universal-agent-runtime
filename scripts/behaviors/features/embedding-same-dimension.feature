@api @knowledge @embedding
Feature: Same-dimension embedding change requires regeneration
  Scenario: Regenerate a retained document after switching to a different model of the same width
    Given a disposable knowledge base indexed with the original embedding model
    When I change the active embedding model to a different model with the same vector dimension
    Then the old knowledge index refuses retrieval despite the equal dimension
    When I regenerate the knowledge index from its retained document
    Then retrieval returns the original private fact using the new embedding space
