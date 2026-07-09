## Context

The admin shell exposes a dedicated **A2UI Schema Testing** page (`frontend/src/admin/A2uiTestingPage.tsx`) wired in through two registration points:
- `frontend/src/admin/admin-shell.tsx` — an `"a2ui-testing"` member in the nav-id union and a corresponding nav item.
- `frontend/src/pages/admin-page.tsx` — an import of `A2uiTestingPage` and an `"a2ui-testing"` entry in the page/route map.

This is a mechanical removal of a developer-time surface. The genuine A2UI runtime pipeline — schema store (`a2ui-schemas-store.ts`), API service (`a2ui-api.ts`), the `use-a2ui-schemas` hook, chat artifact rendering (`a2ui-artifact-block.tsx`), and live A2UI surfaces in the runtime console (`runtime-console-page.tsx`) — is independent of the testing page and remains in place.

## Goals / Non-Goals

**Goals:**
- Remove the A2UI testing page component and both of its registration points so it no longer ships in or is reachable from the production admin/runtime-console navigation.
- Leave the frontend build and typecheck clean, with no dangling references to the removed `"a2ui-testing"` id or `A2uiTestingPage` symbol.

**Non-Goals:**
- No change to any real A2UI product feature (store, service, hook, artifact rendering, runtime-console surfaces).
- No backend, API, provider-routing, or realtime-ingestion change.
- No redesign of the admin shell or runtime console beyond removing the single nav entry.

## Decisions

- **Delete the component file outright** rather than gating it behind a feature flag. It is a testing playground with no production role; a flag would leave dead code and a dead nav path. This matches the phase's "retire from prod" intent.
- **Remove both registration points in the same change** (nav union+item in `admin-shell.tsx`, import+route in `admin-page.tsx`). Removing only one would leave either an unreachable route or a nav item that renders nothing / fails typecheck.
- **Re-anchor the "replayed A2UI surfaces visible" requirement to the runtime console** (handled in the spec delta), since that behavior is delivered by `runtime-console-page.tsx`, not the retired page — so retiring the page does not regress live A2UI protocol visibility.

## Risks / Trade-offs

- **Risk:** a stale reference to `"a2ui-testing"` or `A2uiTestingPage` elsewhere causes a typecheck/build break. **Mitigation:** grep for both identifiers across `frontend/src` before finishing and run `pnpm run build` / typecheck as the change's verification step; the earlier survey found references confined to the three known files.
- **Trade-off:** developers lose the standalone schema-preview playground. **Accepted:** schema listing/preview/validation remain available programmatically via the store and API service, and operators observe live A2UI surfaces through the runtime console — the operationally relevant path.
