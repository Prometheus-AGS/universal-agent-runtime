# embedded-admin-surface

## ADDED Requirements

### Requirement: Embedded runs can seed an empty session from host history

The embedded runtime SHALL accept a host-supplied conversation history when
starting a run and, when the resolved session holds no messages, replay that
history into the session before the current user message — so a cold-started
conversation (whose durable history lives in the host, not the in-process
store) still presents prior turns to the model. When the session already holds
messages, the supplied history SHALL NOT be replayed, so repeated turns never
duplicate context.

#### Scenario: A cold-started session is seeded from supplied history

- **WHEN** a run starts for a conversation whose in-process session is empty,
  with a seed history of a prior user turn and assistant reply, plus a new user
  input
- **THEN** the request sent to the model contains the prior user turn, the prior
  assistant reply, and the new input

#### Scenario: A warm session is not re-seeded

- **WHEN** a second run starts on the same session with the same seed history
- **THEN** the prior turns appear only once in the model request (the warm
  session is not re-seeded)
