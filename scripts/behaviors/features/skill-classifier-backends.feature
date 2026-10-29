@api @skills @settings
Feature: Each intent-classifier backend selects and uses a skill
  Scenario Outline: The <backend> classifier picks the target skill and its private rule shapes the answer
    Given a disposable agent with owned target and decoy skills
    And skill matching is switched to the intent classifier with the <backend> backend
    When I send the target's discriminating prompt through a real run
    Then the run reports its backend's legacy_classifier method with the target accepted over the decoy
    And the answer follows the private rule stored only in the target skill

    Examples:
      | backend        |
      | rules          |
      | hybrid         |
      | localembedding |
      | llm            |
      | wasm           |
