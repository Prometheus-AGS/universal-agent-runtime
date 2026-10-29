@api @openai @structured-output
Feature: Structured output follows the supplied schema
  # E05. The schema carries a fresh discriminator that the prompt never mentions,
  # so plain prompt compliance cannot satisfy it.
  Scenario: A response_format JSON schema shapes the final answer
    Given a disposable agent on a structured-output model with a schema holding a fresh discriminator
    When I ask for a fresh calculation through /v1/chat/completions with that response_format
    Then the final answer validates against the exact schema, carries the discriminator and the correct result
