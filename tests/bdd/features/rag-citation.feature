Feature: RAG-augmented chat responses carry numbered citation markers
  As a user asking a question against a knowledge-base-scoped agent
  I want the retrieved sources to show up as numbered [1], [2] citation markers
  So that I can hover a marker and see exactly which document it came from

  Scenario: Numbered citation markers appear for a RAG-augmented response
    Given I have ingested a knowledge base document containing "The secret onboarding code is ZEBRA-42."
    And a fresh conversation with an agent scoped to that knowledge base
    When I ask "What is the secret onboarding code mentioned in the handbook?"
    Then the assistant responds with content containing "ZEBRA-42"
    And a RAG citation source badge is shown in the transcript
    And hovering the first citation badge reveals its source document

  Scenario: No citation markers when no knowledge base is attached
    Given a fresh conversation with an agent that has no knowledge bases attached
    When I send the message "What is 2 plus 2?"
    Then the assistant responds with content containing "2 plus 2 is 4"
    And no RAG citation source badge is shown in the transcript
