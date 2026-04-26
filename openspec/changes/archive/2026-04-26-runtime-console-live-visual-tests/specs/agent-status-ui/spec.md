## ADDED Requirements

### Requirement: Runtime console status affordances are visually covered
The agent status UI SHALL have targeted visual coverage when status affordances appear inside runtime console or chat-adjacent operator surfaces.

#### Scenario: Status surface renders without console layout regression
- **WHEN** Playwright opens a runtime console surface that displays agent processing status or status empty states
- **THEN** the status content MUST be visible
- **AND** it MUST NOT overlap primary navigation or page headings.

#### Scenario: Status coverage preserves existing chat status behavior
- **WHEN** runtime console visual tests are added
- **THEN** the existing chat status requirements for thinking, tool execution, searching, completion, and transition behavior MUST remain unchanged.
