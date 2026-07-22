@ui @shell
Feature: KnowMe width-adaptive shell navigation
  As a user on any device width
  I want top-level destinations in a rail on wide layouts and a bottom bar on narrow ones
  So that navigation is always reachable and consistent with the KnowMe standard

  Scenario: Wide viewport renders the rail with destinations and the readiness lane
    Given the app is open at 1440 pixels wide
    Then the rail shows destinations "Chat", "Admin", and "About"
    And the active destination is "Chat"
    And the readiness lane reports "Ready" and "Embedded · local"
    And I capture a screenshot named "shell-rail-1440"

  Scenario: Narrow viewport renders the bottom navigation bar
    Given the app is open at 390 pixels wide
    Then the bottom navigation bar shows destinations "Chat", "Admin", and "About"
    And the rail is hidden
    And I capture a screenshot named "shell-bottombar-390"

  Scenario: Narrow navigation reaches the About destination
    Given the app is open at 390 pixels wide
    When I tap the "About" destination in the bottom bar
    Then the About page is shown
    And I capture a screenshot named "shell-about-390"
