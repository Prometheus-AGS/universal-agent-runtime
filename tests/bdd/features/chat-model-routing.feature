Feature: Chat with provider/model configuration affecting the answering model
  As a user configuring which agent (and therefore which provider/model) answers
  I want the outgoing LLM request's model field to actually change
  So that provider/model configuration is a real routing decision, not just display state

  # Agent config uses "provider/model" (matches the UI's displayed label);
  # the wire-level request to a provider with an explicit base_url carries
  # just the bare model id (liter-llm registry.rs: routing is already
  # resolved via the base_url, so no provider prefix is added) -- confirmed
  # via stub-llm's /_stub/requests introspection.
  Scenario: Model configuration changes the answering model
    Given two agents "BDD Agent Aurora" (model "openai/gpt-5.4-mini") and "BDD Agent Borealis" (model "bdd-provider-b/claude-5-haiku") exist
    And a fresh conversation using "BDD Agent Aurora"
    When I send the message "Which agent is answering?"
    Then the outgoing request's model field is "gpt-5.4-mini"
    When I switch the active agent to "BDD Agent Borealis"
    And I send the message "Which agent is answering?"
    Then the outgoing request's model field is "claude-5-haiku"
