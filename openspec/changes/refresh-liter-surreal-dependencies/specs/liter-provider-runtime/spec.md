## ADDED Requirements

### Requirement: Liter provider responses remain incrementally streamed
The runtime SHALL return a normalized model stream as soon as Liter establishes the upstream response stream and SHALL normalize chunks as they arrive instead of buffering the full completion before returning control to the tool loop.

#### Scenario: Post-tool model completion exceeds the stream-start timeout
- **WHEN** Liter establishes a response stream within the configured stream-start timeout and the completion continues beyond that timeout
- **THEN** the runtime emits the available normalized events incrementally and does not fail the run merely because the full completion takes longer than the stream-start timeout
