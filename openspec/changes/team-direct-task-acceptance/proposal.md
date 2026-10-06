## Why

The actual C14 packaged Work operation rejected coding package preflight with HTTP 422 (`collaboration_invalid`). Its direct coordinator TeamDefinition declares `allowedWorkflows: []`, but draft.2 currently requires a workflow even though direct team tasks and coordinator delegation already use independent bounded admission.

## What Changes

- Permit an empty required workflow allowlist only for `coordinator-within-binding`; retain at least one immutable workflow reference for `operator` mode.
- Clarify that an empty list authorizes no workflow launch and confers no task, tool, model or workspace authority.
- Record the preserved failure and defer acceptance to the lead-owned actual native rebuild and packaged operation.

## Capabilities

### Modified Capabilities

- `collaboration-package-catalog`: mode-specific TeamDefinition workflow allowlist shape.

## Impact

Only draft.2 TeamDefinition schema, its normative README and this repair's OpenSpec artifacts change. No runtime, provider compatibility, realtime state, persistence, credential, admission or UI implementation changes. The UI can proceed past this structural rejection only after the rebuilt runtime accepts the package; later boundaries remain unproven. Parent owns canonical KBD transitions; this worker makes none. No dependency or tool-workflow changes.
