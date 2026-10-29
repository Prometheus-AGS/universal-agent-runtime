@api @identity @isolation @restart
Feature: External owner contexts isolate runs and conversation history
  # E09. Both owners come from identity endpoints provisioned outside the test;
  # the test sends no credentials.
  Scenario: A second owner can neither touch the first owner's run nor read its conversation
    Given a disposable agent and owner A's private fact in a fresh session
    When owner B reads owner A's run and its events and asks to cancel it
    Then owner B is refused each operation and owner A's run is unchanged
    When owner B asks about the fact in the same public session ID
    Then owner B's answer comes from its own empty partition without owner A's fact
    When the owned UAR restarts with its existing store
    And owner A asks about the fact in the same session without replaying it
    Then owner A still recalls the fact while owner B's run stays invisible to owner A
