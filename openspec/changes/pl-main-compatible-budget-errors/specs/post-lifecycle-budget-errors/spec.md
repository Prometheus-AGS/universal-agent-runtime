## ADDED Requirements

### Requirement: Budget admission preserves its refusal origin

Root, tool and child budget admission SHALL retain the reviewed closed budget error classification through the host boundary. A denied admission SHALL occur before downstream provider or tool effects. Unrelated provider failures and disconnected transports SHALL retain their distinct classifications.

#### Scenario: Root tool or child limit refuses work

- **WHEN** a root, tool or child admission exceeds its enforced budget
- **THEN** the caller receives the retained budget refusal class and the denied downstream effect count remains zero

#### Scenario: Eligible work and unrelated failure controls

- **WHEN** below-limit work is admitted or a provider failure or transport disconnect occurs independently of admission
- **THEN** eligible work can execute and unrelated failures are not relabeled as budget refusal

### Requirement: Streaming budget refusal has one compatible terminal representation

The OpenAI streaming adapter SHALL deliver the reviewed budget error representation and selected termination sequence exactly once after a streaming budget refusal. It SHALL preserve successful chunk framing, SHALL NOT change an already-started HTTP status and SHALL NOT add citation extensions under this change.

#### Scenario: Budget refusal after streaming starts

- **WHEN** an admitted streaming request reaches a budget refusal after its HTTP stream has started
- **THEN** the selected client parser observes one terminal budget error with the reviewed code, message and termination sequence without an attempted HTTP status replacement

#### Scenario: Successful stream control

- **WHEN** an ordinary successful stream completes
- **THEN** its existing successful chunk framing and completion behavior remain available without a synthetic budget error
