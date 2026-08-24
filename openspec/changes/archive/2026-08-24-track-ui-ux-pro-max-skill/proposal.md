## Why

UI/UX Pro Max is installed locally and already named by the repository's mandatory UI/UX routing instructions, but its canonical payload, tool links, and installer lock are untracked. A fresh checkout therefore receives an instruction to consult a skill that is not actually present.

## What Changes

- Track the repository-owned UI/UX Pro Max skill payload and upstream MIT license while keeping other `.agents/` state machine-local.
- Track the installer lock and existing cross-tool skill links so supported agents resolve one canonical payload.
- Update the durable UI/UX skill roster with the canonical local path, query contract, verified upstream source, and current catalog counts.
- Verify skill data integrity, a representative React search, tracked-path resolution, and the existing AGENTS/CLAUDE routing references.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `uar-uiux-skill-routing`: Require the always-consult UI/UX Pro Max roster entry to resolve to a tracked repository skill with reproducible metadata and usable tool entry points.

## Impact

Affected areas are `.agents/skills/ui-ux-pro-max/`, selected tool skill links, `skills-lock.json`, `.gitignore`, the UI/UX roster, and OpenSpec artifacts. Runtime UX, provider compatibility, public APIs, application types, and realtime state are unchanged. The previous KBD run remains terminal; this workflow-only change does not reopen product phase state.
