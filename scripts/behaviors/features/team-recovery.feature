@api @team @restart
Feature: A team recovers after cancellation and restart without duplicate work
  Scenario: Cancelled work stays honest across a restart and only permitted work resumes to a dependent conclusion
    Given the saved math and writing specialists form an owned recovery team
    And a math tally, a long volunteer note and a dependent join are submitted
    When the math tally settles with its artifact
    And I cancel the volunteer note once its attempt is running
    Then the cancelled note reports an honest non-success state
    When I restart the owned UAR
    Then the settled tally artifact, attempt and model calls are byte-identical after the restart
    And the cancelled note is still reported honestly with no new model calls
    When I ask the team to recover only what its recovery design permits
    Then recovery re-dispatches nothing and the dependent of the cancelled note stays blocked
    When I submit an explicit replacement note and a replacement join over the settled tally artifact
    Then the replacement join reaches a legitimate conclusion from both artifacts
    And no settled tally work or cancelled note attempt was silently repeated
