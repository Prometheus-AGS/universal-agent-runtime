# Migrating to 1.0

- Legacy `/api/chat` session methods were removed because that server route is
  disabled. Use `client.chat().complete(...)` or `.stream(...)`.
- Run creation now requires the compiled agent `artifact`, matching
  `/api/uar/runs`.
- `Error::Api` now preserves `error_code` and `request_id` and implements
  `miette::Diagnostic`.
- Knowledge configuration and document lifecycle methods are fully typed.
- Ingestion accepts optional metadata.
- Streaming returns `EventStream<Item = Result<StreamEvent>>` and supports
  `last_event_id` replay for runs.
