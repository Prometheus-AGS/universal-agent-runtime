@api @context
Feature: Bounded context preserves information needed later
  Scenario: Reduce oversized conversation context and answer a dependent follow-up
    Given a disposable agent with a small context budget and keep-first-and-latest strategy
    When I give it a fresh reference and quantities to remember
    And I add oversized unrelated context and ask about the earlier information
    Then the model request fits the budget after actual context reduction
    And the follow-up correctly uses the original information
