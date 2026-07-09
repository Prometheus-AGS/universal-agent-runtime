## Why

`RuntimeProtocolsPage` (`frontend/src/admin/pages/runtime-console-page.tsx`)
renders three live-data panels — Recent AG-UI Events, Model Routing, A2UI
Surfaces — backed by `RuntimeAgUiEvent`, `RuntimeModelRouteDecision`, and
`RuntimeA2uiSurface` entities that the backend never populates. The page
silently shows generic `EmptyRuntimeState` placeholders indistinguishable
from "no activity yet" — an operator has no way to tell "this feature isn't
built" from "nothing has happened recently." This is the "Protocols page
gating" carryover item, open since `uar-production-readiness-gaps`
(2026-06-02) and reconfirmed still open by this phase's assessment.

User decision (AskUserQuestion, 2026-07-09): apply the cheap fix — an
honest "not yet implemented" gating disclosure — rather than building real
backend AG-UI/model-route/A2UI event emission (a genuinely new
cross-cutting feature, out of proportion with closing a facade gap). Real
backend wiring is deferred to a future phase if wanted.

## What Changes

- Add a reusable `NotWiredBanner` (or equivalent) disclosure component,
  matching this project's existing `AdminError`/`AdminEmptyInline`
  admin-state pattern, that clearly states a section is not yet wired to
  live backend data — distinct in appearance from the generic
  "no activity yet" empty state.
- Apply it to all three of `RuntimeProtocolsPage`'s live-data panels
  (Recent AG-UI Events, Model Routing, A2UI Surfaces), replacing their
  `EmptyRuntimeState` fallback.
- No backend changes. No removal — the page's static protocol-surface
  cards (Anthropic REST, OpenAI REST, MCP endpoints) are real and stay
  as-is; only the three dynamic-but-never-populated panels are gated.

## Impact

- Affected capability: `runtime-console-ux` (new delta — this capability
  did not exist as an OpenSpec spec prior to this change; the assessment
  and prior phases tracked "Protocols page gating" only as a carryover
  note, never as a spec).
- Affected code: `frontend/src/admin/pages/runtime-console-page.tsx`
  (and its shared admin-state components file, if the new banner is added
  there).
