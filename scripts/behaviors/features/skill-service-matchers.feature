@api @skills
Feature: Service embedding and hybrid skill matchers select and use a skill
  Scenario: The service embedding matcher selects the semantic target and uses its rule
    Given a semantic target skill, an unrelated decoy skill and a disposable agent restricted to both
    When I send the discriminating prompt under the service embedding matcher
    Then the run names skill_service.embedding, accepts only the target and answers with the target's private rule

  Scenario: The service hybrid matcher selects the semantic target and uses its rule
    Given a semantic target skill, an unrelated decoy skill and a disposable agent restricted to both
    When I send the discriminating prompt under the service hybrid matcher
    Then the run names skill_service.hybrid, accepts only the target and answers with the target's private rule

  Scenario: The service llm matcher selects the semantic target and uses its rule
    Given a semantic target skill, an unrelated decoy skill and a disposable agent restricted to both
    When I send the discriminating prompt under the service llm matcher
    Then the run names skill_service.llm, accepts only the target and answers with the target's private rule
