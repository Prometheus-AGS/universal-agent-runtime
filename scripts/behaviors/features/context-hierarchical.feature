@api @context @hierarchical
Feature: Hierarchical context produces real summary tiers
  Scenario: Early and middle turns become long-term facts and a mid-term summary that preserve the answer
    Given a disposable agent using hierarchical context with one short-term turn
    When I give an early private fact and then a later working objective in separate turns
    And I ask a follow-up that needs both without repeating them
    Then real long-term facts and mid-term summary calls reduced the earlier turns within their budgets
    And the answer still uses both the early fact and the working objective
