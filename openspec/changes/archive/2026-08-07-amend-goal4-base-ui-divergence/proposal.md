## Why

Goal 4 still names shadcn as the frontend primitive owner even though operator decision D1 retained Base UI and recorded that choice as an explicit override of the KnowMe UI/UX standard §6.1 and §6.3. The contradiction must be removed before downstream UI changes are assessed against the phase goals.

## What Changes

- Amend Goal 4 to name Base UI as the owner of general controls, navigation, overlays, and sidebars.
- Add a canonical, self-contained frontend design-authority document that records the D1 override, rationale, scope, precedence, provenance, and the requirements that remain unchanged.
- Define the `frontend-design-authority` capability so future UI work can resolve authority conflicts without treating the override as compliance with the vendored standard.

## Capabilities

### New Capabilities

- `frontend-design-authority`: Defines the authoritative frontend design sources, their precedence, and the recorded Base UI divergence from the KnowMe standard.

### Modified Capabilities

None.

## Impact

- **Runtime UX:** Downstream UI work will be reviewed against one coherent component-ownership decision: Base UI-backed local wrappers for general controls, navigation, overlays, and sidebars.
- **Provider compatibility:** No provider, model-routing, API, or transport behavior changes.
- **Realtime state:** No AG-UI, A2UI, entity-graph, store, or persistence behavior changes.
- **KBD workflow state:** C-01 is recorded through canonical KBD change transitions when work starts and completes; the existing D1 decision remains the rationale of record.
- **Affected artifacts:** Phase Goal 4, a new `docs/ui-design-authority.md`, and the new OpenSpec capability. No runtime code or dependencies change.
