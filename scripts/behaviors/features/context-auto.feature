@api @context @auto
Feature: Auto context resolves for the selected model
  Scenario: Auto resolves to the documented strategy for the model window and keeps the recent fact
    Given a disposable agent using the auto context strategy on a model with a known context window
    When I continue a seeded conversation longer than the resolved message budget
    Then the reduction receipt shows the strategy the documented rule derives from that window
    And the answer uses the recent fact and cannot see the oldest one
