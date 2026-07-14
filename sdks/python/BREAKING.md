# Python SDK 1.0 breaking changes

- Chat now uses the live OpenAI-compatible `/v1/chat/completions` route. Pass a string or a sequence of typed `Message` values; the removed alpha session-chat API targeted disabled legacy routes.
- Runs now require the runtime `artifact` object and expose the full `/api/uar/runs` lifecycle.
- Ingestion accepts a file path because the server endpoint is multipart, not JSON content.
- Errors derive from `UarError`; HTTP failures expose status and request ID through `ApiError`.
