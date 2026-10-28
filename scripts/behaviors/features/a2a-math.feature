@api @a2a
Feature: Prompt a saved agent through A2A
  Scenario: The saved math agent completes an A2A task with the correct answer
    Given the saved math agent is available through A2A
    When I send it a fresh arithmetic question through A2A SendMessage
    Then the completed A2A task contains its saved identity and correct answer
