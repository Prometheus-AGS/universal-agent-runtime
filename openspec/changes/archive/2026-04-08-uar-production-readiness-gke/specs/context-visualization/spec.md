## ADDED Requirements

### Requirement: Display context window usage in chat UI
The chat interface SHALL display a visual indicator of current context window utilization showing tokens used vs. available.

#### Scenario: Token usage bar
- **WHEN** a conversation has used 50,000 of 128,000 available tokens
- **THEN** the UI displays a progress bar at ~39% with the token counts

#### Scenario: Threshold warning
- **WHEN** token usage exceeds the trigger threshold (e.g., 85%)
- **THEN** the progress bar changes color to amber/warning

#### Scenario: Strategy indicator
- **WHEN** a context management strategy is active (e.g., SlidingWindow)
- **THEN** a label shows the active strategy name

### Requirement: Context compression event notification
The UI SHALL notify users when context compression occurs.

#### Scenario: Compression notification
- **WHEN** the context manager removes messages to fit the token budget
- **THEN** the UI displays an inline notification showing messages removed and tokens saved
