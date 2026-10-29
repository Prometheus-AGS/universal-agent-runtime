@api @skills
Feature: Below-threshold or ambiguous skills stay inactive
  Scenario: Neither a weak match nor an unresolved tie loads a skill implicitly
    Given a disposable agent and owned target and tied decoy skills under implicit-only matching
    When I send a prompt that matches the target skill only below the acceptance threshold
    Then no skill is accepted or activated and the target's score is below the threshold
    When I send a prompt that matches both tied decoy skills equally
    Then no skill is accepted or activated because the tie is within the margin
    And no skill's private instructions reached either answer
