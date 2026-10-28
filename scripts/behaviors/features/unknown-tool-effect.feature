@api @mcp
Feature: An unknown tool effect is not silently repeated
  Scenario: A committed MCP write whose response is lost stays uncertain and is not repeated
    Given an owned MCP record peer that loses record_write responses is permitted for one disposable agent after an owned restart
    When I ask the agent to store a private value and approve only its first write
    Then the peer committed the write exactly once and the runtime reports the outcome as unknown
    When I reconnect to the finished run's stream
    Then the replay still shows the unknown outcome and the peer effect count is unchanged
