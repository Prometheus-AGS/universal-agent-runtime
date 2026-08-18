@ui @cross-screen @certification
Feature: Cross-screen identity, memory, and agent boundaries
  The browser must prove the behaviors that span individual product screens.

  Scenario: Default and orchestrator agents return exact attributed answers
    When I ask the default and orchestrator agents their deterministic questions
    Then both exact answers are visible and the orchestrator contribution is attributed

  Scenario: A verified JWT reaches protected credentials and anonymous access fails
    When I compare verified and anonymous credential requests
    Then the verified credential request succeeds and the anonymous request is rejected

  Scenario: User, agent, and conversation memory remain private to their owner
    When two verified subjects address the same session memory and knowledge identifiers
    Then the owner sees every resource and the other subject sees none
