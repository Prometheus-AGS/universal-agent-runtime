## Why

The Provider (set-default, remove) and Agent (patch, delete) migrations introduced a snapshot-based optimistic-rollback pattern. Three near-identical inlined implementations now live in the codebase; the pattern will be extracted to a `useOptimisticPatch` helper in a future phase. Before that extraction, we want a regression test pinned to the existing inline implementation so the helper extraction can be done with confidence.

This test exercises the Provider `setDefault` path because it has the simplest snapshot shape (`ProviderMeta` singleton with one field).

## What Changes

Author `frontend/src/admin/pages/__tests__/providers-set-default-rollback.test.tsx`:

- Stub `services/providers-api::setDefaultProvider` with `vi.mock(...)` to throw.
- Seed the graph with two providers + `ProviderMeta:current` whose `default_id` is `"p1"`.
- Extract the page's `setDefault(id)` body into an exported helper if it isn't already testable; if extraction is too invasive, invoke through the rendered component via `userEvent.click(...)`.
- Call `setDefault("p2")`.
- `await waitFor(() => { /* error settled */ })`.
- Assert `useGraphStore.getState().entities["ProviderMeta"]["current"].default_id === "p1"` — i.e. the optimistic flip rolled back.

## Acceptance

- Test passes.
- Removing the rollback logic (commenting out the `upsertEntity(..., previousDefault)` line) makes the test fail.
