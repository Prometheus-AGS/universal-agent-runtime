@api @acp
Feature: Prompt a saved agent through ACP
  Scenario: The saved math agent answers an ACP prompt correctly
    Given an ACP session targets the saved math agent
    When I send a fresh arithmetic question through the ACP stream
    Then the completed ACP run contains the saved identity and correct answer
