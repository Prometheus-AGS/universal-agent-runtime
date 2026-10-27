@api @settings @models @restart
Feature: REST model configuration is effective and durable
  Scenario: A saved default model, output limit and session override survive a UAR restart
    Given the original provider configuration and two verified model bindings are recorded
    When I save model B as the default with a smaller output limit through the REST API
    And I save model A as the model override for one fresh session
    Then a stale default-provider revision and an invalid session model are refused without changing saved state
    And prompts without a requested model run on B for the default session and A for the override session
    And model B stops at its saved output limit although the request asks for more
    When UAR restarts with the existing store
    Then the saved default, output limit and session override are unchanged after restart
    And prompts without a requested model again run on B for the default session and A for the override session
    And model B again stops at its saved output limit although the request asks for more
