@ui @cross-screen @certification
Feature: Cross-screen identity, memory, and agent boundaries
  The browser must prove the behaviors that span individual product screens.

  Scenario: Default and orchestrator agents return exact attributed answers
    When I ask the default and orchestrator agents their deterministic questions
    Then both exact answers are visible and the orchestrator contribution is attributed

  Scenario: A verified JWT reaches protected credentials and anonymous access fails
    When I compare verified and anonymous credential requests
    Then the verified credential request succeeds and the anonymous request is rejected

  Scenario: Global, agent, and user memory work while user state remains private
    When two verified subjects address scoped memory and the same session and knowledge identifiers
    Then all memory levels resolve and the owner sees private resources while the other subject sees none
