# Tiktoken Estimation Specification

## Purpose

Define tokenizer-aware OpenAI token estimation, fallback estimation for other models, and per-message protocol overhead accounting.

## Requirements

### Requirement: Accurate token estimation for OpenAI models
The token service SHALL use tiktoken-compatible tokenization for OpenAI models instead of character-based estimation.

#### Scenario: OpenAI model token count
- **WHEN** estimating tokens for text "Hello, world!" with model `openai/gpt-4o`
- **THEN** the count matches tiktoken's `cl100k_base` encoding result (4 tokens)

#### Scenario: Non-OpenAI model fallback
- **WHEN** estimating tokens for a model without a known tokenizer (e.g., `groq/llama-3`)
- **THEN** the system falls back to a 4:1 character-to-token ratio estimate

### Requirement: Token estimation includes message overhead
The token service SHALL account for per-message overhead tokens (role markers, separators).

#### Scenario: Message overhead counted
- **WHEN** estimating tokens for a ChatML-formatted message
- **THEN** the estimate includes ~4 tokens of overhead per message (role, separators)
