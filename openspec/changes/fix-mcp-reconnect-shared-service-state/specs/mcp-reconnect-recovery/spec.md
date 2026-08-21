## Purpose

Defines fail-closed MCP transport recovery that remains effective across independent authorized runtime requests without replaying uncertain tool operations or widening tool access.

## ADDED Requirements

### Requirement: Replacement transports survive registry view boundaries
After an MCP tool call fails because its server transport crashed or timed out, UAR SHALL make any successfully reconnected transport available to later independent authorized calls without requiring a UAR restart.

#### Scenario: Filtered requests observe the replacement
- **WHEN** one authorized request fails an MCP call and UAR successfully reconnects that server
- **THEN** a separately created authorized request view MUST use the replacement transport for its next call
- **AND** the later call MUST NOT reuse the failed transport

#### Scenario: Merged registry views observe the replacement
- **WHEN** an authorized merged registry view exists before another view successfully replaces a failed server transport
- **THEN** the merged view's next call to that server MUST use the replacement transport

### Requirement: Failed MCP operations are not replayed
UAR SHALL report the failed MCP tool operation as unsuccessful and SHALL NOT automatically execute that operation again while reconnecting the transport.

#### Scenario: Process crash is fail-closed
- **WHEN** an MCP subprocess exits during a tool operation
- **THEN** the calling run MUST receive exactly one unsuccessful tool-result event for that operation
- **AND** the operation MUST appear exactly once at the MCP process boundary
- **AND** a later independent operation MAY succeed through a replacement process

#### Scenario: Tool timeout is fail-closed
- **WHEN** an MCP tool operation exceeds the configured runtime timeout
- **THEN** the calling run MUST receive exactly one unsuccessful tool-result event for that operation
- **AND** the timed-out operation MUST appear exactly once at the MCP process boundary
- **AND** a later independent operation MAY succeed through a replacement process

### Requirement: Reconnect recovery preserves authorization
Sharing replacement transport state across registry views SHALL NOT add servers, MCP tools, or native tools that a view's resolved policy excluded.

#### Scenario: Excluded server remains unavailable
- **WHEN** a server transport is replaced through an authorized view and another view excludes that server
- **THEN** the excluding view MUST continue to omit the server and all of its tools

#### Scenario: Excluded tool remains unavailable
- **WHEN** a view permits an MCP server but excludes one of that server's tools and the server transport is replaced elsewhere
- **THEN** the excluded tool MUST remain unavailable in that view
- **AND** permitted tools on the replacement transport MUST remain callable
