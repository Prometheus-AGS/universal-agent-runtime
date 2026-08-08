## Why

Assessment M1: the AG-UI spec mode lacks STATE_SNAPSHOT, MESSAGES_SNAPSHOT
and RAW, and remaps ToolStart to TOOL_CALL_END - gaps against the official
vocabulary the operator requires fully emitted.

## What Changes

- Emit the three missing official event types where the protocol calls for them.
- Correct the ToolStart mapping; extend golden schema tests.

## Capabilities
### New Capabilities
- `agui-spec-parity`

## Impact
SSE mapping, run replay, golden tests.
