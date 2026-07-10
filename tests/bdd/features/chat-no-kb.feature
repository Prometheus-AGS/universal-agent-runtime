Feature: Chat with no knowledge base attached
  As a user with no knowledge base configured for my agent
  I want a plain chat response with no retrieval-related UI
  So that I can trust the chat works standalone, without requiring a KB

  Scenario: Plain chat with no knowledge base
    Given a fresh conversation with an agent that has no knowledge bases attached
    When I send the message "What is 2 plus 2?"
    Then the assistant responds with content containing "2 plus 2 is 4"
    And no knowledge-base retrieval indicator is shown in the transcript
