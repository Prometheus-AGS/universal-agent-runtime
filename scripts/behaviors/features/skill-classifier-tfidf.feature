@api @skills @settings
Feature: Intent-classifier tfidf selects and uses a skill
  Scenario: The tfidf classifier picks the target skill and its private rule shapes the answer
    Given a disposable agent with owned target and decoy skills
    And skill matching is switched to the intent classifier with the tfidf backend
    When I send the target's discriminating prompt through a real run
    Then the run reports the legacy_classifier.tfidf method with the target accepted over the decoy
    And the answer follows the private rule stored only in the target skill
