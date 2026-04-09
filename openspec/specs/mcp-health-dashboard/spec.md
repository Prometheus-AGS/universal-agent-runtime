## ADDED Requirements

### Requirement: Admin page showing MCP server status
The admin dashboard SHALL include a page listing all configured MCP servers with their connection status, tool count, and last-seen timestamp.

#### Scenario: All servers healthy
- **WHEN** the user navigates to the MCP Health admin page and all servers are connected
- **THEN** each server shows a green status indicator, tool count, and transport type (stdio/HTTP)

#### Scenario: Server disconnected
- **WHEN** an MCP server process has exited or HTTP endpoint is unreachable
- **THEN** the server shows a red status indicator with the error message

#### Scenario: Refresh status
- **WHEN** the user clicks "Refresh" on the MCP Health page
- **THEN** the system re-checks all server connections and updates the display
