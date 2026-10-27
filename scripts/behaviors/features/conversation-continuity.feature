@api @continuity
Feature: Continue a conversation with the saved mathematics agent
  Scenario: A follow-up uses the first exchange without the client repeating it
    Given the saved math agent and a fresh conversation
    When I ask it to add two fresh numbers
    Then its first answer is correct and uses its saved identity
    When I ask a follow-up using only references to the previous exchange
    Then it answers using the remembered numbers and previous result
