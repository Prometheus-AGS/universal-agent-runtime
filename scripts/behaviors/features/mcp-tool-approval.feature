@api @tools @approval @mcp
Feature: Real MCP tool discovery, approval and result use
  Scenario: Discover an owned MCP peer, approve a write and read, then deny a write
    Given an owned MCP record peer is permitted for one disposable agent after an owned restart
    When I start the owned MCP peer on loopback HTTP
    Then the peer serves its record tools over MCP
    Given a disposable agent that may use only the peer's record tools
    When I ask the agent to store and read back a private value and approve its MCP calls
    Then the peer stored the value once and the answer reports the value and revision the peer returned
    When I ask the agent to overwrite the value and deny the MCP call
    Then the peer record is unchanged and the answer reports that the write was not performed
