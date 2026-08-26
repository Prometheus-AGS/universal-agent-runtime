## ADDED Requirements

### Requirement: Customer documentation explains prompt caching completely
The documentation site SHALL provide a dedicated prompt-caching guide and cross-links from provider configuration, cost, observability, and troubleshooting guidance. The guide MUST describe configuration, precedence, persistence and fallback, APIs, metrics, cache-write and cache-read costs, fixed ephemeral or provider-default retention, provider differences, and internal calls that do not inherit user policy without claiming that UAR controls OpenAI automatic caching.

#### Scenario: An operator configures prompt caching
- **WHEN** an operator follows the prompt-caching guide
- **THEN** the operator can distinguish global, user, session, and request settings and determine which value is effective

#### Scenario: An operator estimates provider behavior
- **WHEN** an operator compares Anthropic and OpenAI prompt caching
- **THEN** the guide explains explicit Anthropic ephemeral cache controls, OpenAI provider-managed eligibility, relevant usage metrics, and provider-specific cost semantics

#### Scenario: Prompt caching fails or appears inactive
- **WHEN** an operator follows troubleshooting links
- **THEN** the docs identify authorization, persistence fallback, unsupported-model, cache-eligibility, TTL, and metrics checks without promising a cache hit
