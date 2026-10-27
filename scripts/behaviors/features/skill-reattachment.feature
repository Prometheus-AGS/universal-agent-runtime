@api @skills @context @compaction
Feature: Active skill survives compaction within its reattachment budget
  Scenario: A small permitted skill is reattached after compaction while an oversized one cannot widen the budget
    Given a small skill reattachment allowance and two attached skills, one small and one oversized
    When a conversation long enough to be compacted continues with a task needing the small skill's rule
    Then the conversation was genuinely compacted
    And the small skill body was reattached within the allowance and the oversized body did not widen it
    And the answer follows the small skill's private rule
