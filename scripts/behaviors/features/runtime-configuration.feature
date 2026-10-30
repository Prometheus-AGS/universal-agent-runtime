Feature: The runtime honors the model, embedding and budget configuration it is given

  Each scenario starts its own UAR on the flint-forge PostgreSQL 18 image, so the
  configuration under test is the only configuration in play.

  Scenario: UAR_LLM__MODEL outranks LLM_MODEL, which outranks the config file (#419)
    Given a dedicated UAR whose config file names model "openai/gpt-4.1" with LLM_MODEL "openai/gpt-4o-mini" and UAR_LLM__MODEL "openai/gpt-5.4-mini"
    Then UAR loads model "openai/gpt-5.4-mini"
    When UAR restarts without UAR_LLM__MODEL
    Then UAR loads model "openai/gpt-4o-mini"
    When UAR restarts without LLM_MODEL
    Then UAR loads model "openai/gpt-4.1"

  Scenario: Memory uses the Mistral embedding API it is configured with (#420)
    Given a dedicated UAR with memory using the Mistral embedding API against a stub embedder
    Then UAR reports the Mistral embedding API at startup
    When I save a memory record
    Then the stub embedder received a "mistral-embed" request
    When UAR restarts with the embedding API "bogus"
    Then UAR refuses to start and names the unsupported embedding API

  Scenario: An output cap keeps a budgeted agent within its per-turn token limit (#385)
    Given a dedicated UAR with the stub model
    And an agent with a per-turn token limit of 450000
    When I ask that agent with max_tokens 1000
    Then the agent answers
