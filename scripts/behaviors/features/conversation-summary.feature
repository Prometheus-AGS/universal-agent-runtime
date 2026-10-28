@api @context @summarization
Feature: Conversation summarization preserves key facts
  Scenario: Recall booking details after actual summarization
    Given a disposable agent using conversation summarization
    When I provide booking facts within a lengthy conversation turn
    And I ask a follow-up without repeating the booking facts
    Then a genuine summary replaces the fact-bearing turn and preserves the answer
