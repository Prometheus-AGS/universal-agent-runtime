## Why

`RuntimeCockpitPage` (`frontend/src/admin/pages/runtime-console-page.tsx`)
renders a Provider Health panel and a Memory Activity panel backed by
`RuntimeProviderHealth` / `RuntimeMemoryEvent` entities the backend never
populates — permanently empty, same facade class as the Protocols page.
Confirmed by this phase's assessment; same fix-vs-remove tension flagged
alongside `resolve-runtime-protocols-page-facade`.

User decision (AskUserQuestion, 2026-07-09, covering both changes): apply
the cheap fix — an honest "not yet implemented" disclosure — rather than
building real backend Provider Health / Memory Activity event emission.
Real backend wiring (e.g. reusing `McpHealthPage`'s existing
`/api/uar/mcp/health`-style signal for Provider Health specifically, noted
as a possible low-effort path in `plan.md`) is deferred to a future phase
if wanted — out of scope for this narrowly-scoped disclosure fix.

## What Changes

- Reuse the `NotWiredRuntimeState` disclosure component added by
  `resolve-runtime-protocols-page-facade` (same file,
  `runtime-console-page.tsx`) for `RuntimeCockpitPage`'s Provider Health and
  Memory Activity panels, replacing their `EmptyRuntimeState` fallback.
- The rest of `RuntimeCockpitPage` (Live Runs, Execution Timeline, and the
  four stat tiles) is backed by real, populated entities and is untouched.
- No backend changes, no removal.

## Impact

- Affected capability: `runtime-console-ux` (extends the requirement added
  by `resolve-runtime-protocols-page-facade` to cover these two panels).
- Affected code: `frontend/src/admin/pages/runtime-console-page.tsx`.
