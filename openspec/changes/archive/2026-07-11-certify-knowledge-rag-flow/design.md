## Context

The Knowledge page currently renders useful create, upload, search, delete, loading, and empty states, but its compatibility hook owns service I/O and durable domain state. That bypasses the required Component → Hook → Store → Service → API direction and prevents deterministic rollback/retry tests.

## Decisions

### A knowledge store owns I/O and graph reconciliation

A dedicated Zustand store owns knowledge-base and document loads, mutations, upload progress, search results, errors, and optimistic rollback. The hook becomes a subscription/action façade. Dialogs, selected knowledge base, drag state, file input, and search text remain local presentation state.

### Retry reuses the selected browser file

The API does not persist original document bytes after ingestion. The store therefore retains the `File` object for the current browser session, keyed by knowledge base and filename. Retrying a failed document removes the failed record and re-uploads the exact bytes with rollback if either operation fails. After reload, the UI requires the user to select the source file again rather than pretending a retry is possible.

### Existing visual language remains stable

This is behavioral hardening, not a redesign. UI/UX Pro Max, Impeccable audit/critique guidance, frontend-design, and Vercel React guidance were consulted. The implementation preserves repository tokens and focuses on announced errors, explicit progress, actionable empty/failure states, keyboard-safe controls, parallel independent work, and narrow store subscriptions. The deterministic Impeccable detector reported no markup anti-patterns on the Knowledge page.

## Risks / Trade-offs

- Browser `File` objects are intentionally session-only → retry availability is explicit and never presented after the bytes are unavailable.
- Entity graph and store projections could drift → every authoritative load reconciles both, and realtime graph changes are projected by the hook.
- Async indexing may outlive the page → polling remains a presentation coordinator and calls the store's idempotent document load action.
