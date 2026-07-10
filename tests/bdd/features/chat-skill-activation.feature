Feature: Chat with a skill activated mid-conversation
  As a user whose message matches a configured skill's trigger
  I want the frontend to visibly show which skill activated
  So that I can trust the system is actually applying the skill, not just replying generically

  Scenario: Skill visibly activates mid-conversation
    Given a fresh conversation with an agent bound to a skill triggered by the keyword "diagnostics"
    When I send the message "Please run diagnostics on the system."
    Then the assistant responds with content containing "Diagnostics complete"
    And a skill-activation indicator for "BDD Diagnostics Skill" is shown in the transcript
