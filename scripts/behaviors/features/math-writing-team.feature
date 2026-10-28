@api @team
Feature: Mathematics and writing agents collaborate on a project
  Scenario: Concurrent calculation and editorial work merge into one article
    Given the saved math and writing agents and executable team API are available
    When I create a team containing those two specialists
    And I submit a project with independent math and editorial tasks and a dependent article
    And UAR executes the math and editorial tasks concurrently
    Then both specialists produce their own successful results
    When the writer receives both results to finish the project
    Then the final article incorporates the math result and the editorial work
