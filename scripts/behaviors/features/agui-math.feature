@api @agui
Feature: Stream an answer from a saved agent through AG-UI
  Scenario: The saved math agent streams a correct answer through AG-UI
    Given the saved math agent is available for AG-UI
    When I send it a fresh arithmetic question through AG-UI
    Then its correlated AG-UI stream contains the saved identity and correct answer
