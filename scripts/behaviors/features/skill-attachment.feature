@api @skills @attachment
Feature: Explicit skill attachment works in catalog mode
  Scenario: An attached skill's private rule drives the answer
    Given a disposable agent and an eligible skill whose triggers cannot match the request
    When I send a fresh request that attaches the skill
    Then the skill is activated as an attachment rather than by matching
    And the answer follows the skill's private calculation rule
