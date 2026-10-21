# Spec Delta

## MODIFIED Requirements

### Requirement: Tool execution preserves application and uncertain outcomes

**Stable task:** R4. Tool outcomes SHALL distinguish success, application failure, transport failure and ambiguous external outcome. Unknown effects SHALL NOT advance success dependencies or be automatically replayed. MCP outcome classification SHALL use the MCP response contract only; native tool payloads with coincidental `isError`-named data SHALL retain their native outcome semantics. A later recovery SHALL NOT rewrite an immutable earlier receipt.

#### Scenario: Application error

- **WHEN** MCP returns `isError=true` inside a successful transport response
- **THEN** ToolEnd, admission and durable receipts report application failure with original error content

#### Scenario: Ambiguous external effect

- **WHEN** transport ends after an effect may have occurred without definitive result
- **THEN** outcome remains unknown, success dependencies stay blocked and automatic replay is prohibited

#### Scenario: Explicit false or omitted MCP error indicator

- **WHEN** an MCP tool result is delivered through a successful transport response with `isError=false` or with `isError` omitted
- **THEN** the runtime records the response according to its MCP application-result semantics without treating it as an application failure

#### Scenario: MCP protocol or transport failure

- **WHEN** an MCP response is a protocol error, a timeout, or an interrupted transport without a definitive result
- **THEN** the runtime records a typed failure or unknown outcome according to observed effect certainty, retaining uncertainty whenever an effect may have occurred
- **AND** it does not convert the outcome into success or replay an effect automatically

#### Scenario: Receipt is immutable after recovery

- **WHEN** a later independent attempt succeeds after an MCP application failure, transport failure, or unknown effect outcome
- **THEN** the earlier ToolEnd, admission, and durable receipt retain their original typed outcome
- **AND** the later attempt is recorded independently

#### Scenario: File-configured MCP tools survive first-boot settings bootstrap

- **WHEN** the runtime starts with a configured MCP server file and only the untouched empty `mcp.servers` bootstrap default exists
- **THEN** the configured tool is present in the connected catalog for model selection and truthful outcome recording
- **AND** an explicitly API-written empty `mcp.servers` map remains authoritative over the file configuration
