@api @conversation @concurrency @restart
Feature: Concurrent conversation turns survive restart
  Scenario: Recall facts from both overlapping turns after restarting UAR
    Given a disposable agent and a fresh conversation for overlapping turns
    When I send two fact-bearing turns concurrently in that conversation
    And UAR restarts after both concurrent turns finish
    And I ask for both sets of facts without replaying either turn
    Then the answer retains the facts from both concurrent turns
