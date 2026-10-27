@ui @orchestrator @routing
Feature: The browser shows orchestrator agent selection before the answer
  Scenario: A writing task and then a math task show each selected specialist first
    Given the saved math and writing specialists and an orchestrator conversation in the browser
    When I send a writing task in that conversation through the browser
    Then the browser shows the writing specialist selection before its specialist answer
    When I send a math task in the same conversation through the browser
    Then the browser shows the math specialist selection before its specialist answer
    And each selected specialist matches execution with no duplicate answer or run
