## Why

Repository-wide OpenSpec validation is blocked by the active `implement-opencode-suggestions` change because several requirement deltas use non-normative wording. This blocks the runtime console validation-hardening phase from reaching its final archive gate.

## What Changes

- Repair or retire invalid OpenSpec deltas in `openspec/changes/implement-opencode-suggestions/` so active change validation can pass.
- Preserve the intended product requirements where possible by converting non-normative requirement/scenario text to SHALL/MUST wording.
- If any stale requirement delta is no longer relevant, remove or archive it through the OpenSpec workflow rather than leaving invalid active specs.
- Add a narrow validation workflow record so future KBD execution can prove that global OpenSpec change validation was restored.
- No runtime API, frontend UI, provider-routing, or realtime event behavior changes are intended.

## Capabilities

### New Capabilities

- `openspec-validation-hygiene`: Active OpenSpec changes can be kept valid as part of KBD phase closure, with invalid requirement deltas repaired or retired before final archive readiness.

### Modified Capabilities

- None.

## Impact

- Affected OpenSpec areas include `openspec/changes/implement-opencode-suggestions/`, this cleanup change, and any main specs touched if deltas are synced or archived.
- Runtime UX impact: none directly; the work unblocks runtime console archival by restoring spec validation confidence.
- Provider compatibility impact: none; no provider credentials or live model calls are required.
- Realtime state impact: none; no runtime event or entity graph behavior changes are intended.
- KBD workflow state must be updated as this change advances, with `.kbd-orchestrator/` remaining the source of truth and Surreal Memory remaining a secondary mirror when available.
