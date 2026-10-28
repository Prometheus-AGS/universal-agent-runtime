@api @team
Feature: Team follow-up reassembles child context and dependencies
  Scenario: A premature join is refused and the next round consumes the parents' actual artifacts
    Given the saved math and writing specialists form an owned team
    And a first-round math task, a private-fact writer task and a dependent join are submitted
    When I submit the dependent join before its parents finish
    Then the premature join is refused with no attempt and no model dispatch
    When the parent tasks finish in their own child threads
    And I submit the legitimate next round with the parents' actual artifacts
    Then the next round remembers the private first-round fact through the child thread
    And the next round consumed exactly the parent artifacts once
