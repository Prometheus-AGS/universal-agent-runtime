@api @context @structural
Feature: Structural context strategies remove the intended history
  Scenario: Sliding-window retains the intended recent groups
    Given a disposable agent using a sliding window of 4 messages
    When I continue a seeded 10-message conversation that names an old and a recent depot code
    Then the reduction receipt shows old groups left the prepared input
    And the answer uses the recent fact and cannot see the old one

  Scenario: Truncate-middle actually removes the middle
    Given a disposable agent using truncate-middle keeping the first 2 and last 2 messages
    When I continue a seeded 10-message conversation that names first, middle and latest depot codes
    Then the reduction receipt shows the middle removed and the first and latest groups retained
    And the answer uses the first and latest facts and cannot see the middle one
