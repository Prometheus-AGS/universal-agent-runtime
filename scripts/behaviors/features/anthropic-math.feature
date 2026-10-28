@api @anthropic
Feature: Prompt a saved agent through the Anthropic-compatible API
  Scenario: The saved math agent returns the correct answer through Messages
    Given the saved math agent is available for an Anthropic-compatible request
    When I send it a fresh arithmetic question through the Messages API
    Then the Anthropic response completes with the saved specialist identity and correct answer
