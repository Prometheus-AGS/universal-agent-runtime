> **STATUS: ABANDONED / SUPERSEDED (2026-07-09).** This change was never
> implemented. During its apply, a concurrent `git pull` brought in commit
> `d37abfd` _"feat(a2ui): upgrade A2uiTestingPage to a real live round-trip
> tester"_, which upgrades the page this change proposed to retire into a real
> operational tester — invalidating the premise below. Per operator decision the
> retirement is abandoned; the page and its nav/route wiring remain in place.

## Why

The admin dashboard ships a dedicated **A2UI Schema Testing** playground page (`A2uiTestingPage`) whose purpose is developer-time exercising of built-in A2UI artifact schemas. As the frontend converges on a compact live runtime operations console, this bespoke testing surface is a non-operational developer tool that clutters the production admin navigation and duplicates concerns better served by the runtime console's live A2UI surface view. It should be retired from the production build while the genuine A2UI runtime features (schema store, API service, artifact rendering, and runtime-console A2UI surfaces) remain untouched.

## What Changes

- Remove the dedicated A2UI testing page component `frontend/src/admin/A2uiTestingPage.tsx` from the production admin surface.
- Remove the `"a2ui-testing"` navigation entry (id union member and nav item) from `frontend/src/admin/admin-shell.tsx` so the page is no longer reachable from the admin/runtime-console navigation.
- Remove the `A2uiTestingPage` import and its `"a2ui-testing"` route entry from `frontend/src/pages/admin-page.tsx`.
- **Preserved (explicitly NOT changed):** live A2UI surfaces in the runtime console (`runtime-console-page.tsx`), the A2UI schema store (`a2ui-schemas-store.ts`), the A2UI API service (`a2ui-api.ts`), the `use-a2ui-schemas` hook, and chat artifact rendering (`a2ui-artifact-block.tsx`). These are product features, not the testing playground.

## Capabilities

### New Capabilities
<!-- None. This is a removal of an existing surface; no new capability is introduced. -->

### Modified Capabilities
- `a2ui-testing-ui`: The requirements describing a **dedicated admin page for A2UI artifact testing** (schema listing, form preview, test submission, custom-schema validation) and its **responsive reachability from the runtime console navigation** are removed — the testing page is retired from production. The requirement that **replayed A2UI surfaces are visible as live runtime protocol state** is retained but re-anchored to the runtime console (its real home), since that behavior is delivered by the runtime console, not the retired testing page.

## Impact

- **Affected code:**
  - Deleted: `frontend/src/admin/A2uiTestingPage.tsx`
  - Modified: `frontend/src/admin/admin-shell.tsx` (remove `"a2ui-testing"` nav id + nav item)
  - Modified: `frontend/src/pages/admin-page.tsx` (remove import + route entry)
- **Runtime UX:** The admin/runtime-console navigation loses the "A2UI Testing" item; operators exercise A2UI surfaces through the live runtime console instead. No end-user chat behavior changes.
- **Provider compatibility:** None. This is a frontend-only admin surface removal; no backend, API, or provider routing is affected.
- **Realtime state:** Unchanged. Live A2UI surface ingestion into the entity graph and its display in the runtime console are preserved.
- **Dependencies:** None added or removed. The A2UI store/service/hook remain in use by the runtime console and chat.
- **KBD workflow state:** This change is tracked as change 3/9 of phase `uar-production-ready-uiux-2026-07`; `progress.json` and the waypoint are advanced by `/kbd-apply` on task completion. No other KBD state changes required.
