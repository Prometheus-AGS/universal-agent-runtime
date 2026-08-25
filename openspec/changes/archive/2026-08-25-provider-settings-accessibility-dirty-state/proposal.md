## Why

Provider settings repeat visually labeled controls without complete programmatic associations, and the page does not clearly expose unsaved, loading, success, or error state. Operators can also refresh a stale remote baseline or leave the page while whole-provider drafts are pending, creating preventable overwrite or data-loss risk.

## What Changes

- Associate every provider control with provider-specific accessible names and connect help or invalid text to the affected control.
- Add appropriate polite status and assertive error semantics for loading and save outcomes.
- Disable Save while clean, expose visible dirty-provider feedback, and keep failed-save drafts intact.
- Protect dirty drafts by disabling Refresh until they are saved and by warning at the browser-unload boundary.
- Make provider fields stack at narrow widths while retaining the incumbent two-column desktop layout.
- Add focused accessibility, dirty-state, refresh-protection, live-status, and responsive-structure coverage.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-configuration-surfaces`: Require accessible provider settings controls, honest live state, dirty-draft protection, and responsive provider editing.

## Impact

- Runtime UX: provider cards become distinguishable in assistive technology, save state becomes explicit, and unsaved drafts are protected from refresh and browser exit.
- Provider compatibility: provider values, payloads, and save semantics remain unchanged.
- Realtime state: the existing provider settings hook remains authoritative; Refresh is gated while a local whole-provider draft could overwrite a refreshed remote baseline.
- Code: provider settings panel, optional shared primitive accessibility/status props, settings hook state exposure if needed, and focused frontend tests.
- Dependencies: none.
- KBD: register this as the second change, dependent on completion of `provider-model-search`.
