@api @skills
Feature: Catalog skill activation by the model and its count limit
  Scenario: The model explicitly activates a catalog skill whose body alone holds the rule
    Given catalog activation mode and a disposable agent with one eligible private catalog skill
    When I ask for a task that needs the skill without attaching it
    Then the first call sees only the catalog, the model activates the skill, and its private rule reaches the answer

  Scenario: The activation count limit refuses a second skill and keeps the first usable
    Given catalog activation mode and a disposable agent with max_active one and two eligible private skills
    When I attach the first skill and the model attempts to activate the second
    Then the second activation is refused with limit_reached, its body never loads, and the first skill's rule answers
