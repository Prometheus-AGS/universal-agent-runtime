# Spec Delta

## ADDED Requirements

### Requirement: Supported streams require protocol completion evidence

Every supported streaming route SHALL preserve incremental framing and require the selected protocol's terminal evidence before reporting successful completion.

#### Scenario: EOF without terminal

- **WHEN** a supported provider connection closes without its required terminal event
- **THEN** the runtime reports an incomplete/error outcome rather than successful completion

#### Scenario: Arbitrary byte boundaries

- **WHEN** valid events including multibyte characters and tool arguments cross transport chunks
- **THEN** the selected production route reconstructs the same events and succeeds only after valid terminal evidence


