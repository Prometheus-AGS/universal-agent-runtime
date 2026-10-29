@api @skills @policy @restart
Feature: Skill scope and lower-scope policy cannot widen access
  Scenario: Lower scopes only narrow the parent skill selection, and a scoped disable is durable
    Given skills A, B and C with private rules and an agent whose parent selection is A and B with B denied
    When a conversation selects A and C and the run attaches A, B and C
    Then only A is eligible and active, and the answer follows A's rule
    When a conversation selects no skills
    Then no skill is eligible or active
    When A is disabled for a second conversation that selects A
    Then that conversation never loads A
    And a third conversation that selects A uses A's private rule
    And after an owned restart the second conversation's disable of A is still saved
