## ADDED Requirements

### Requirement: Anthropic SSE decoding is independent of chunk boundaries
The native Anthropic driver SHALL produce the same normalized event sequence for a given SSE byte stream however it is split into chunks, and SHALL fail without a successful terminal on malformed JSON, invalid UTF-8 or a missing `message_stop`.

#### Scenario: Every two-chunk split
- **WHEN** a fixed SSE fixture with multibyte text, tool input deltas, usage and `message_stop` is fed as one chunk, at every two-chunk split, and one byte at a time
- **THEN** every feeding yields the same normalized events and exactly one `Done`.
