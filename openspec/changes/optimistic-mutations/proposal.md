## Why

Once every entity flows through the graph + SSE realtime spine, high-frequency mutations (toggles, name edits, enable/disable flips) feel sluggish if the UI waits for the server roundtrip + SSE event before reflecting the new state. Optimistic updates close that perception gap by patching the graph immediately and letting the SSE event reconcile in the background.

We deliberately keep optimistic updates **opt-in per mutation** rather than blanket-default, because creates and deletes are higher-stakes and a botched optimistic delete looks worse than a 250 ms wait.

## What Changes

Apply `useEntityCRUD({ optimistic: true })` (or the equivalent patch pattern) to high-frequency mutations only:

- **Skill toggle** (`POST /api/skills/{id}/toggle`).
- **Agent enable/disable**.
- **Provider set-default** (`POST /api/uar/providers/{id}/default`).
- **Setting field edits** (`PUT /api/uar/settings/{key}`).
- **KB rename**.

All other CRUD (create new agent, delete provider, upload document, etc.) remains non-optimistic — wait for the server response, then for the SSE event to confirm.

Rollback policy: on server rejection, `useEntityCRUD` reverts the optimistic patch and shows a toast. The SSE channel reconfirms the authoritative state within ~200 ms regardless.

## Acceptance

- Clicking a skill toggle flips the UI immediately (≤16 ms); SSE event arrives within 200 ms and confirms.
- Forcing a server rejection (e.g. delete a Builtin skill) shows a toast and the UI reverts.
- Non-optimistic mutations (create agent, delete KB) show the same loading/spinner as today and update once the server responds.
