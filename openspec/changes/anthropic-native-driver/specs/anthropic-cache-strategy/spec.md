## ADDED Requirements

### Requirement: System Prompt Cache Annotation
CacheStrategy SHALL annotate system prompt blocks with `cache_control: {"type": "ephemeral"}` when caching is enabled for system prompts.

#### Scenario: System prompt meets token threshold
- **WHEN** `cache_system_prompt` is true and the system prompt block exceeds the `min_tokens_to_cache` threshold (default 1024 tokens)
- **THEN** CacheStrategy MUST add `cache_control: {"type": "ephemeral"}` to the system prompt content block.

#### Scenario: System prompt below token threshold
- **WHEN** `cache_system_prompt` is true but the system prompt block is below the `min_tokens_to_cache` threshold
- **THEN** CacheStrategy MUST NOT add `cache_control` to the system prompt block.

### Requirement: Tool Definition Cache Annotation
CacheStrategy SHALL annotate the last tool definition with `cache_control: {"type": "ephemeral"}` when caching is enabled for tools.

#### Scenario: Tools array caching enabled
- **WHEN** `cache_tools` is true and the tools array is non-empty
- **THEN** CacheStrategy MUST add `cache_control: {"type": "ephemeral"}` to the last tool definition in the `tools` array only.

#### Scenario: Tools caching disabled
- **WHEN** `cache_tools` is false
- **THEN** CacheStrategy MUST NOT add `cache_control` to any tool definition.

### Requirement: Conversation Turn Cache Annotation
CacheStrategy SHALL annotate the N most recent conversation turns with `cache_control: {"type": "ephemeral"}`.

#### Scenario: Sufficient conversation history
- **WHEN** `cache_conversation_turns` is set to 4 and the message history contains 10 or more user/assistant pairs
- **THEN** CacheStrategy MUST add `cache_control: {"type": "ephemeral"}` to the last message in each of the 4 most recent user/assistant turn pairs.

#### Scenario: Conversation history shorter than configured turns
- **WHEN** `cache_conversation_turns` is set to 4 but the message history contains only 2 user/assistant pairs
- **THEN** CacheStrategy MUST annotate all 2 available turn pairs and not error.

### Requirement: Minimum Token Threshold
CacheStrategy SHALL respect the minimum 1024 token threshold for cacheable blocks.

#### Scenario: Block below minimum threshold
- **WHEN** a conversation turn block contains fewer tokens than `min_tokens_to_cache`
- **THEN** CacheStrategy MUST skip cache annotation for that block.

#### Scenario: Block at or above minimum threshold
- **WHEN** a conversation turn block contains tokens equal to or exceeding `min_tokens_to_cache`
- **THEN** CacheStrategy MUST annotate that block with `cache_control: {"type": "ephemeral"}`.

### Requirement: Per-Block-Type Configuration
CacheStrategy SHALL allow each block type (system, tools, conversation turns) to be independently enabled or disabled.

#### Scenario: Only system prompt caching enabled
- **WHEN** `cache_system_prompt` is true, `cache_tools` is false, and `cache_conversation_turns` is 0
- **THEN** CacheStrategy MUST annotate only the system prompt block and leave tools and conversation turns unannotated.

### Requirement: Cache Usage Reporting
CacheStrategy SHALL report cache write and read token counts in Usage events.

#### Scenario: Cache hit on repeated request
- **WHEN** the Anthropic API response includes `cache_read_input_tokens` and `cache_creation_input_tokens` in the usage object
- **THEN** the driver MUST emit a `NormalizedEvent::Usage` event that includes `cache_read_input_tokens` and `cache_creation_input_tokens` fields.

#### Scenario: No cache interaction
- **WHEN** the Anthropic API response usage object contains zero values for cache token fields
- **THEN** the driver MUST still emit a `NormalizedEvent::Usage` event with cache token fields set to zero.
