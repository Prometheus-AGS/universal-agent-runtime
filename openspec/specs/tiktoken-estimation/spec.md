# Tiktoken Estimation Specification

## Purpose

Define tokenizer-aware OpenAI token estimation, fallback estimation for other models, and per-message protocol overhead accounting.

## Requirements

### Requirement: Accurate token estimation for OpenAI models
The token service SHALL use tiktoken-compatible tokenization keyed by model, selecting `o200k_base` or `cl100k_base` when the model maps to a known encoding, and SHALL use `cl100k_base` as the fallback for models without a known tokenizer instead of character-based estimation.

#### Scenario: OpenAI model token count
- **WHEN** estimating tokens for text "Hello, world!" with model `openai/gpt-4o`
- **THEN** the count matches tiktoken's encoding result for that model

#### Scenario: Non-OpenAI model fallback
- **WHEN** estimating tokens for a model without a known tokenizer (e.g., `groq/llama-3`)
- **THEN** the system counts with `cl100k_base` and labels the estimate as a fallback in telemetry

#### Scenario: No character-ratio path remains
- **WHEN** any context reducer or budget check computes a token count
- **THEN** it calls the token service; no `len/4` estimator exists in the runtime

### Requirement: Token estimation includes message overhead
The token service SHALL account for per-message overhead tokens (role markers, separators).

#### Scenario: Message overhead counted
- **WHEN** estimating tokens for a ChatML-formatted message
- **THEN** the estimate includes ~4 tokens of overhead per message (role, separators)
