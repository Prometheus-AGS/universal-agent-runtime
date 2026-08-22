Feature: Chat with a knowledge base enabled, retrieval influencing the response
  As a user with a knowledge base attached to my agent
  I want retrieved document content to actually reach the LLM call
  So that answers are grounded in my ingested documents, not generic

  Scenario: Retrieval-influenced response
    Given I have ingested a knowledge base document containing "The secret onboarding code is ZEBRA-42."
    And a fresh conversation with an agent scoped to that knowledge base
    When I ask "What is the secret onboarding code mentioned in the handbook?"
    Then the assistant responds with content containing "ZEBRA-42"
    And the outgoing request's system prompt actually contains "The secret onboarding code is ZEBRA-42."
    And a RAG citation source badge is shown in the transcript
    And hovering the first citation badge reveals its source document
