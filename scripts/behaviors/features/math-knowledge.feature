@api
Feature: A saved mathematics agent answers from its assigned knowledge base
  Scenario: Retrieve specialized wheel-graph mathematics with source provenance
    Given the saved mathematics agent and a real embedding model are available
    When I create a specialized mathematics knowledge base and upload a research note
    And the research note finishes indexing
    And I assign that knowledge base to the saved mathematics agent
    And I ask the mathematics agent about the private research profile
    Then the answer contains the correct mathematics and a citation to the retrieved note
