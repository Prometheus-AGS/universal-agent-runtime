@api @agents @orchestrator @writing
Feature: Route a writing request to a saved writing specialist
  As a user
  I want the orchestrator to choose a writer from my registered agents
  So that writing tasks are handled by the writer rather than the mathematics specialist

  Scenario: Save a writer and let intent classification route a blog request
    Given the running UAR contains the saved mathematics specialist
    When I register a persistent blog and article writing agent
    Then the writing agent is saved alongside the mathematics specialist
    When I ask the orchestrator to write a community garden blog post
    Then intent classification selects the writing agent instead of the mathematics specialist
    And the writing agent returns a blog draft using its saved byline
