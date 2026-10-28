@api @anthropic @default-agent
Feature: Anthropic Messages without an agent id use the protected default agent
  Scenario: A JSON Messages request with agent_id omitted is answered by the default agent
    Given an Anthropic-compatible request that omits agent_id
    When I send a fresh calculation and then a dependent follow-up as JSON Messages requests
    Then both answers are correct and complete and the runs used the default agent

  Scenario: A streamed Messages request with agent_id omitted is answered by the default agent
    Given an Anthropic-compatible request that omits agent_id
    When I send a fresh calculation and then a dependent follow-up as streamed Messages requests
    Then both streams deliver incremental text and complete with correct answers from the default agent
