@api @settings @restart
Feature: REST setting survives a UAR restart
  Scenario: Persist a changed user preference through restart
    Given the original prompt caching preference is recorded
    When I change that preference through the REST API
    And UAR restarts with the existing settings store
    Then the REST API returns the same saved preference after restart
