@api @skills
Feature: Lower-scope skill policies cannot widen the parent selection
  Scenario: An inherited turn policy preserves the parent's selected skill set
    Given catalog presentation, skills A, B and C, and an agent that selects A and denies C
    When a turn that inherits the skill policy asks for a task that needs A's private rule
    Then the effective selection is exactly A, B and C never load, and A's rule answers

  Scenario: An auto turn policy chooses within the parent's narrowed set
    Given catalog presentation, skills A, B and a tempting decoy C, and an agent that selects A and B
    When a turn with an auto skill policy asks for a task that C's description matches best
    Then the effective selection is exactly A and B, C is absent, and A is activated and answers
