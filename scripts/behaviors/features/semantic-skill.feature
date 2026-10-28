@api @skills @semantic
Feature: Detect skills by meaning rather than exact keywords
  Scenario: A paraphrased delivery request selects and uses the matching skill
    Given a disposable agent with delivery and gardening skills under semantic matching
    When I describe a delivery task without any exact skill keywords
    Then semantic matching selects and activates the delivery skill instead of gardening
    And the answer follows the privately stored delivery recipe
