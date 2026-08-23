# Progressive Summarization Specification

## Purpose

Define model-driven context compression that preserves important conversation state when a configured token threshold is exceeded.

## Requirements

### Requirement: ProgressiveSummarization uses LLM for context compression
The ProgressiveSummarization context strategy SHALL call an LLM to generate conversation summaries when the context window exceeds the trigger threshold, replacing older messages with a summary message.

#### Scenario: Context exceeds threshold
- **WHEN** conversation token count exceeds `trigger_threshold` (e.g., 85% of max_tokens) and strategy is `ProgressiveSummarization`
- **THEN** the system calls the configured summarization model to summarize older messages, replaces them with a summary message, and preserves the system prompt and recent messages

#### Scenario: Summarization model fallback
- **WHEN** no `summarization_model` is configured
- **THEN** the system uses the current conversation model for summarization

#### Scenario: Summary preserves key information
- **WHEN** a summarization is generated
- **THEN** the summary message includes key decisions, tool results, and user preferences from the summarized messages
