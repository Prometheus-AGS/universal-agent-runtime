Feature: Chat with agent selection and mid-session switching
  As a user with more than one agent available
  I want switching the active agent to actually change who answers
  So that agent selection isn't just cosmetic

  Scenario: Switching agent changes the answering agent
    Given two agents "BDD Agent Aurora" (model "openai/gpt-5.4-mini") and "BDD Agent Borealis" (model "bdd-provider-b/claude-5-haiku") exist
    And a fresh conversation using "BDD Agent Aurora"
    When I send the message "Which agent is answering?"
    Then the assistant responds with content containing "Agent Aurora"
    When I switch the active agent to "BDD Agent Borealis"
    And I send the message "Which agent is answering?"
    Then the assistant responds with content containing "Agent Borealis"
