@api @skills
Feature: Detect and use the appropriate skill
  Scenario: A parcel request selects the pricing skill and uses its private recipe
    Given a disposable agent and distinct parcel and gardening skills
    When I ask for a parcel quote without naming a skill
    Then the runtime selects and activates the parcel skill instead of gardening
    And the answer uses the recipe and marker stored only in that skill
