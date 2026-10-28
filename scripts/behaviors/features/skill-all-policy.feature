@api @skills
Feature: With skills set to all, the agent finds and uses the right installed skill
  Scenario: A turn with skills mode all answers with the matching skill's private rule
    Given an installed private crate-quote skill and an unrelated installed skill
    When I ask for an orchard crate quote in a turn whose skill policy is all
    Then the effective skill policy is all and the answer applies the crate-quote skill's private formula
