@ui @theming
Feature: Theme system reachability and Flat 2.0 enforcement
  As a user who prefers a non-default theme
  I want my stored preference applied at boot and a visible way to switch themes
  So that every shipped theme is actually reachable (regression: themes were unreachable)

  Scenario: A stored light preference applies at boot
    Given the stored theme is "light"
    When the app loads
    Then the document theme is "light"
    And I capture a screenshot named "theme-light-boot"

  Scenario: The theme toggle cycles dark to light
    Given the stored theme is "dark"
    And the app is open
    When I activate the theme toggle
    Then the document theme is "light"
    And the toggle reports "Light mode"
    And I capture a screenshot named "theme-toggled-light"

  Scenario: Flat 2.0 — the chat surface draws no visible borders or shadows
    Given the app is open
    Then no element in the chat surface has a visible border
    And no element in the chat surface has a layout shadow
