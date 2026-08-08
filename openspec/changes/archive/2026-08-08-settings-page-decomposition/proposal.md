## Why

The settings feature is correctly owned after C-14a, but its single 3,336-line page still combines navigation, shared controls, schema rendering, fourteen custom panels, generic namespace panels, and the page shell. That size makes behavior-preserving maintenance and the next legacy-retirement step difficult to review safely.

## What Changes

- Split the settings UI into domain-owned modules while preserving the existing route export, navigation inventory, panel behavior, responsive layout, styling, and accessibility semantics.
- Extract shared settings primitives and schema-driven controls into narrowly named internal UI modules.
- Group custom panels by AI/LLM, file processing, infrastructure, governance/agents, and caching/users, with no resulting page or panel module above approximately 600 lines.
- Retain the existing hooks, Zustand orchestration, REST clients, realtime settings-change behavior, provider/model compatibility, JWT gating, and save/reload/error semantics unchanged.
- Add focused composition and contract coverage for the decomposed panel registry and navigation behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-configuration-surfaces`: add the maintainable settings decomposition and behavior-preservation contract.

## Impact

- Affected code is limited to `frontend/src/features/settings/ui/`, its focused tests, and narrow settings feature exports if required.
- Runtime UX remains visually and behaviorally compatible; this change introduces no new controls, routes, dependencies, provider/model behavior, or realtime state mechanism.
- Backend, REST payloads, authentication semantics, persistence, AG-UI/A2UI, and the entity graph are unchanged.
- Canonical KBD C-14b is already recorded in progress and will transition to complete before archive.
