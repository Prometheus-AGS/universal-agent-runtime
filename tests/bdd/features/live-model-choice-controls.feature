Feature: Live model choice, negative controls
  As the people who rely on live model choice
  I want each promise to be checked against an input a wrong build would get wrong
  So that a pass means the behaviour is real and the checks can tell right from wrong

  # Companion to live-model-choice.feature (S1 of phase uar-uiux-architecture-2026-11).
  # One scenario per negative control of SCN-SYS-001 (unknown provider, failed acknowledgement
  # CTL-SYS-001-HELD-500, per-provider button), 003, 007, 011, 012, 013, 014 and 016.
  # The controls of SCN-SYS-015 and SCN-SYS-017 are mutants of the candidate, not scenarios
  # here: M2 (receipt time read from the send time) runs the main feature's second scenario,
  # M1 (receipt label read from the request) runs its last scenario; see design.md "Mutants".
  # Authored, not executed. Same stub providers, per-provider stubs and unique messages
  # as the main feature. BDD runs send no credentials.

  Background:
    Given the stub providers "openai", "bdd-provider-b" and "bdd-provider-c" are configured with models "gpt-5.4-mini", "claude-5-sonnet, claude-5-haiku" and "qwen-5-turbo"
    And the system default model is "openai/gpt-5.4-mini"

  # Control of SCN-SYS-003: agents with no model are not pinned, however they were created.
  Scenario: Agents without a model follow the new default and are not listed as pinned
    Given an agent "BDD Agent Aurora" with the model "openai/gpt-5.4-mini" exists
    And an agent "BDD Agent Dawn" with no model exists
    And a fresh conversation using "Default Assistant"
    When the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    Then the change result does not list "Default Assistant" as pinned
    And the change result does not list "BDD Agent Dawn" as pinned
    When I send the message "S1 c003"
    Then the reply to "S1 c003" came from "bdd-provider-b/claude-5-haiku"

  # Control of SCN-SYS-007: the boot-only setting is not a way to change the default.
  Scenario: A direct write of the default provider setting is refused and changes nothing
    Given a fresh conversation using "Default Assistant"
    When a client writes the setting "llm.default_provider" as "bdd-provider-b" through the settings API
    Then the settings API refuses the write with a message pointing to the live default route
    And the registry default is still "openai"
    And the stored default provider is still "openai"
    When I send the message "S1 c007"
    Then the reply to "S1 c007" came from "openai/gpt-5.4-mini"

  # Control of SCN-SYS-001: an unknown provider cannot become the default.
  Scenario: A request to make an unknown provider the default is refused and changes nothing
    Given a fresh conversation using "Default Assistant"
    When a direct request sets the default provider to "bdd-missing"
    Then the change is refused with a reason mentioning "not found"
    And the registry default is still "openai"
    When I send the message "S1 c001"
    Then the reply to "S1 c001" came from "openai/gpt-5.4-mini"

  # Control CTL-SYS-001-HELD-500 of SCN-SYS-001: an acknowledgement that fails is never confirmed.
  # The held live route is answered with a 500 and never reaches the server, so the registry
  # read-back and the no-further-write check guard only against a fallback write path that
  # does reach the server; they cannot detect the 500 themselves.
  @CTL-SYS-001-HELD-500
  Scenario: A default change whose acknowledgement fails shows no confirmation
    Given a fresh conversation using "Default Assistant"
    When the operator confirms the system default change to "bdd-provider-b/claude-5-haiku" while the acknowledgement is held
    And the held acknowledgement fails with status 500
    Then no confirmation is shown
    And an error says the system default was not changed
    And the system default control still shows "openai/gpt-5.4-mini" as live
    And the console issued no write to the server after the failed acknowledgement
    And the registry default is still "openai"

  # Control of SCN-SYS-001: the per-provider button is a different action; it sets the
  # provider's own default model, so it cannot stand in for the system default change.
  Scenario: The per-provider default button serves the provider's own default model
    Given a fresh conversation using "Default Assistant"
    When I use the per-provider Set as default button for "bdd-provider-b"
    And I send the message "S1 c001 button"
    Then the reply to "S1 c001 button" came from "bdd-provider-b/claude-5-sonnet"
    And the reply to "S1 c001 button" did not come from "bdd-provider-b/claude-5-haiku"

  # Control of SCN-SYS-011: without an override the conversation follows the default.
  Scenario: A conversation whose override was cleared follows the new default
    Given a conversation "Chat X" with the model override "bdd-provider-c/qwen-5-turbo"
    When I clear the model override of this conversation
    And I confirm the change
    And the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    And I send the message "S1 c011"
    Then the reply to "S1 c011" came from "bdd-provider-b/claude-5-haiku"

  # Control of SCN-SYS-012: the preview tells the truth about what the runtime would serve.
  # With the default's provider disabled the runtime answers on the configured fallback,
  # so the preview names that model, never the disabled default.
  Scenario: Clearing an override while the default's provider is disabled previews the fallback
    Given the system default model is "bdd-provider-b/claude-5-haiku"
    And a conversation "Chat X" with the model override "bdd-provider-c/qwen-5-turbo"
    And the provider "bdd-provider-b" is disabled
    When I clear the model override of this conversation
    Then the preview says this conversation will answer with "openai/gpt-5.4-mini" from "configured fallback"
    And the preview does not name "bdd-provider-b/claude-5-haiku"
    When I confirm the change
    And I send the message "S1 c012"
    Then the reply to "S1 c012" came from "openai/gpt-5.4-mini"
    And no outgoing request used the model "claude-5-haiku"
    And the answer's receipt names "openai/gpt-5.4-mini" and the origin "configured fallback"

  # Control of SCN-SYS-013: a build that writes an override to a shared scope moves the other chat.
  Scenario: Two conversations sharing one agent stay independent
    Given the system default model is "bdd-provider-b/claude-5-haiku"
    And the conversation "Chat X" and the conversation "Chat Y" both use the agent "Default Assistant"
    When I switch to the conversation "Chat X"
    And I set the model override of this conversation to "bdd-provider-c/qwen-5-turbo"
    And I switch to the conversation "Chat Y"
    And I send the message "S1 c013"
    Then the reply to "S1 c013" came from "bdd-provider-b/claude-5-haiku"
    And the agent "Default Assistant" still has no model of its own
    And the global policy still has no model

  # Control of SCN-SYS-014: a change made before sending applies to that message.
  Scenario: A change made before sending applies to that message
    Given the stub supports "hold_content"
    And a conversation "Chat Y" with no model override
    When the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    And I send the message "S1 r6 hold"
    Then the reply to "S1 r6 hold" came from "bdd-provider-b/claude-5-haiku"
    And the stub holds no reply

  # Control of SCN-SYS-016: refusals are not blanket; a valid choice is accepted.
  Scenario: A valid choice is accepted and routed
    Given a conversation "Chat X" with no model override
    When I set the model override of this conversation to "bdd-provider-c/qwen-5-turbo"
    And I send the message "S1 c016"
    Then the reply to "S1 c016" came from "bdd-provider-c/qwen-5-turbo"
    And the "Chat X" conversation shows the effective model "bdd-provider-c/qwen-5-turbo" from "this conversation"
