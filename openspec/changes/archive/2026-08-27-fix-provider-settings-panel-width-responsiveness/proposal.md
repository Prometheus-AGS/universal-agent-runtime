## Why

Provider Overrides currently changes its field grid from one to two columns at a viewport breakpoint, so a provider panel constrained inside a wide viewport can retain the desktop composition even when its own width cannot support it. The existing specification already requires the layout to respond to available provider-panel width, but that behavior and its browser-level proof are incomplete.

## What Changes

- Make the provider field composition respond to the available provider-panel width while preserving one column when constrained and exactly two columns when the panel supports the desktop layout.
- Add focused browser verification for a constrained provider panel inside a wide viewport, horizontal-page-overflow prevention, keyboard access, and the desktop two-column state.
- Reuse the installed Tailwind CSS v4 container-query capability and Playwright test stack; add no layout or measurement dependency.
- Preserve the current provider settings hook, draft cache, save/reload behavior, provider compatibility, and realtime reconciliation path.
- Keep native unload-dialog presentation outside this change; the existing contract requires cancellation so the browser can request confirmation, not application-owned prompt copy.
- Update KBD workflow artifacts with the new OpenSpec change, verification evidence, and review receipts while leaving the two archived predecessor changes complete.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-configuration-surfaces`: Clarify and certify that Provider Overrides chooses its one-column or two-column field composition from available provider-panel width, including constrained-panel browser behavior.

## Impact

- Runtime UX: Provider fields remain usable when the settings panel is narrow even if the browser viewport is wide, without horizontal page scrolling or clipped keyboard access.
- Provider compatibility: No provider schema, model inventory, protocol, credential, endpoint, or persistence behavior changes.
- Realtime state: No entity, store, service, transport, subscription, or reconciliation behavior changes.
- Code: The Provider Overrides layout boundary and focused frontend browser tests are affected; component/unit assertions may be adjusted to reflect container-width semantics.
- Dependencies: None. The change adopts installed Tailwind CSS v4 and Playwright capabilities recorded as `cand-001` and `cand-002` in the KBD Analyze candidate contract.
- KBD: The phase gains one follow-up OpenSpec change for the reassessed conformance gap; existing implementation completion records remain historical evidence rather than being relabeled.
