@api @runs @cancellation
Feature: A user can cancel an active run
  Scenario: Cancel a live run and keep prompting afterwards
    Given the saved default agent is available
    When I start a long genuine run and it begins streaming
    And I cancel that run through the cancellation endpoint
    Then the run ends cancelled with no later success
    And its model calls are settled or explicitly uncertain
    And a fresh ordinary prompt still succeeds
