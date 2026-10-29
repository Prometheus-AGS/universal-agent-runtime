@api @startup @retention @restart
Feature: Initial and repeat startup preserve persisted settings and retention removes only expired sessions
  Scenario: Seed once, keep an override across restart, and expire only the aged session
    Given a disposable UAR has completed its initial startup on an empty owned store
    When I save an operator override of a seeded setting
    And the disposable UAR performs a repeat startup on the same store
    Then built-ins were seeded once and the override is still effective
    When I create a terminal session and a fresh control session
    And the host ages only the terminal session past retention
    And a new turn publishes session activity
    Then the retention worker removes only the expired session
    When a second activity signal arrives
    Then nothing else is removed and the protected definitions remain
