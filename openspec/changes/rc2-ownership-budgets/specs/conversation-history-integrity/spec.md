## ADDED Requirements

### Requirement: Concurrent first turns share one conversation
When two turns for the same owner and conversation ID arrive before the conversation exists in memory, UAR SHALL give both turns the same session, so each turn's messages are kept and neither save overwrites the other.

#### Scenario: Recall facts from both overlapping turns after a restart
- **WHEN** two turns that each state a different fact run concurrently in one new conversation, and UAR restarts before a follow-up asks for both facts
- **THEN** the stored conversation holds both turns and the follow-up answers with both facts.
