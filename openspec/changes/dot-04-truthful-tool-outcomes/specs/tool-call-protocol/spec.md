# Spec Delta

## ADDED Requirements

### Requirement: Tool execution preserves application and uncertain outcomes

Tool outcomes SHALL distinguish success, application failure, transport failure and ambiguous external outcome. Unknown effects SHALL NOT advance success dependencies or be automatically replayed.

#### Scenario: Application error

- **WHEN** MCP returns isError=true inside a successful transport response
- **THEN** ToolEnd, admission and durable receipts report application failure with original error content

#### Scenario: Ambiguous external effect

- **WHEN** transport ends after an effect may have occurred without definitive result
- **THEN** outcome remains unknown, success dependencies stay blocked and automatic replay is prohibited


