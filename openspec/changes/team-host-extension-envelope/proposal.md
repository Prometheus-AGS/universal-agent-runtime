## Why
The actual repaired C14 packaged coding-preset-through-work operation reports HTTP422. Source trace identifies a contract mismatch: draft.2 common extension schema requires {required,value} and forbids other envelope keys, while Boss coding documents and UAR host readers use flat {required,version,tools,servers,workspacePath}. Provider execution_profile and outer package/binding DTOs match; no native response body proving the exact rejected stage is yet captured.

## What Changes
Read the canonical host extension envelope through one strict shared TeamHostSelection deserializer. Carry required from the envelope and version/tools/servers/workspacePath from value into existing internal selection. Preserve schema, admission checks, diagnostics and host credentials behavior. Root owns Boss canonical producer and saved-binding comparison repair; this UAR change touches no Boss sources.

## Capabilities
### Modified Capabilities
- trusted-team-host-workspace: canonical draft.2 extension envelope decoding.

## Impact
Only Rust host selection decoding and scoped repair documentation. No schema loosening, alternate authority, model/budget/tool changes, service or dependency changes. Compiler/build/tests/review deferred to lead after complete coordinated producer/reader delivery.
