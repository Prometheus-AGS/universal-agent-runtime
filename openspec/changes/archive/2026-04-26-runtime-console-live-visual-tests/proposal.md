## Why

The runtime console is becoming the primary operations surface for live agent runs, providers, tools, approvals, memory, and protocol debugging, but the current acceptance gates do not prove that the console remains usable across desktop and mobile layouts. This change adds visual and navigation evidence before deeper realtime hardening continues.

## What Changes

- Add Playwright coverage for the `/admin` runtime console shell across desktop and mobile viewports.
- Verify compact navigation, breadcrumbs, detail surfaces, command palette access, and sticky contextual panels render without incoherent overlap.
- Add assertions for provider, protocol, memory, tool approval, and A2UI surfaces so the librefang-inspired information architecture stays navigable.
- Preserve the frontend layering rules: tests may drive browser behavior, but production components continue using components -> hooks -> stores -> services.
- Update KBD workflow state as visual-test artifacts and verification gates complete.
- No breaking changes.

## Capabilities

### New Capabilities

- `runtime-console-visual-verification`: Defines the desktop/mobile runtime console layout, navigation, command palette, and surface-level visual acceptance requirements.

### Modified Capabilities

- `agent-status-ui`: Adds runtime-console visual coverage for live status affordances where agent processing state appears in console or chat-adjacent views.
- `a2ui-testing-ui`: Adds responsive and navigability coverage for A2UI preview/testing surfaces inside the runtime console.
- `tool-approval-workflow`: Adds visual coverage for approval request surfaces so tool approval remains visible and actionable in the runtime console.
- `frontend-validation-gate`: Extends the validation gate to include targeted Playwright runtime-console visual tests for this hardening phase.

## Impact

- Affected code: `frontend/e2e/`, runtime console/admin page selectors, and any minimal test IDs or accessibility labels needed for stable Playwright assertions.
- Affected systems: frontend validation workflow, OpenSpec acceptance, and KBD phase progress tracking.
- Runtime UX impact: operators get regression protection for compact navigation, dense detail panes, command palette access, and responsive console layouts.
- Provider compatibility impact: provider/model routing surfaces become part of the tested console navigation, making provider diagnostics easier to inspect during compatibility work.
- Realtime state impact: this change proves the shell and target surfaces are reachable and visually stable; detailed realtime event replay remains in the follow-up `runtime-event-replay-entity-sync-tests` change.
