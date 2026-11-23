Feature: Live model choice governs the next eligible response
  As an operator choosing the system default model, and a person chatting
  I want a change of model to apply to the next answer without a restart,
  and to see which model answered, why, and since when
  So that I can trust the model I chose is the model that served the reply

  # S1 of phase uar-uiux-architecture-2026-11. Authored, not executed. The step
  # definitions are listed in openspec/changes/uiux-system-model-live-default/design.md
  # under "S1 step phrases". Synthetic stub providers and models only; each provider
  # has its own stub instance, so the request log names the provider that served.
  # Every message sent is unique; a reply is checked against the one stub request
  # carrying that message and against the reply text shown in the transcript.
  # The confirmation check holds the live route's acknowledgement (POST .../default) with
  # the browser's route interception, asserts nothing is confirmed while it is held, then
  # releases it. "the operator changes the system default" always drives the system default picker
  # (save the model, then the live route). The per-provider Set as default button is a
  # different action and never satisfies that step. BDD runs use the deployment's
  # auth-off test configuration and send no credentials.

  Background:
    Given the stub providers "openai", "bdd-provider-b" and "bdd-provider-c" are configured with models "gpt-5.4-mini", "claude-5-sonnet, claude-5-haiku" and "qwen-5-turbo"
    And the system default model is "openai/gpt-5.4-mini"

  Scenario: The operator switches the system default without restarting
    Given a fresh conversation using "Default Assistant"
    And 5 seconds have passed since the system default was set
    When the operator confirms the system default change to "bdd-provider-b/claude-5-haiku" while the acknowledgement is held
    Then no confirmation is shown yet
    And the system default control does not yet show "bdd-provider-b/claude-5-haiku" as live
    And the registry default is still "openai"
    When the acknowledgement is released
    Then the confirmation shows the requested value "bdd-provider-b/claude-5-haiku" and the activation "live"
    And the system default control shows "bdd-provider-b/claude-5-haiku" as live
    When I send the message "S1 r1 after change"
    Then the reply to "S1 r1 after change" came from "bdd-provider-b/claude-5-haiku"
    And the reply to "S1 r1 after change" did not come from "bdd-provider-b/claude-5-sonnet"

  Scenario: A conversation without an override follows the new default and says so
    Given a conversation "Chat Y" with no model override
    And 5 seconds have passed since the system default was set
    When I send the message "S1 r2 before"
    Then the reply to "S1 r2 before" came from "openai/gpt-5.4-mini"
    And the answer's receipt names "openai/gpt-5.4-mini" and the origin "system default"
    And the answer's receipt shows the time the system default became "openai/gpt-5.4-mini"
    When the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    And I send the message "S1 r2 after"
    Then the reply to "S1 r2 after" came from "bdd-provider-b/claude-5-haiku"
    And the answer's receipt names "bdd-provider-b/claude-5-haiku" and the origin "system default"
    And the answer's receipt shows the time the system default became "bdd-provider-b/claude-5-haiku"
    And the answer's receipt does not show the time the system default became "openai/gpt-5.4-mini"

  Scenario: A conversation override keeps answering on its own model
    Given a conversation "Chat X" with the model override "bdd-provider-c/qwen-5-turbo"
    And 5 seconds have passed since the system default was set
    When the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    And I send the message "S1 r3 override"
    Then the reply to "S1 r3 override" came from "bdd-provider-c/qwen-5-turbo"
    And the answer's receipt names "bdd-provider-c/qwen-5-turbo" and the origin "this conversation"
    And the "Chat X" conversation shows the requested model "bdd-provider-c/qwen-5-turbo" and the effective model "bdd-provider-c/qwen-5-turbo" from "this conversation"

  Scenario: Clearing an override shows what the conversation will inherit
    Given the system default model is "bdd-provider-b/claude-5-haiku"
    And a conversation "Chat X" with the model override "bdd-provider-c/qwen-5-turbo"
    When I clear the model override of this conversation
    Then the preview says this conversation will answer with "bdd-provider-b/claude-5-haiku" from "system default"
    When I cancel the change
    And I send the message "S1 r4 cancel"
    Then the reply to "S1 r4 cancel" came from "bdd-provider-c/qwen-5-turbo"
    When I clear the model override of this conversation
    And I confirm the change
    And I send the message "S1 r4 confirm"
    Then the reply to "S1 r4 confirm" came from "bdd-provider-b/claude-5-haiku"
    And the answer's receipt names "bdd-provider-b/claude-5-haiku" and the origin "system default"

  Scenario: Changing one conversation leaves the others alone
    Given the system default model is "bdd-provider-b/claude-5-haiku"
    And a conversation "Chat Y" with no model override
    And a conversation "Chat X" with the model override "bdd-provider-c/qwen-5-turbo"
    When I switch to the conversation "Chat X"
    And I set the model override of this conversation to "openai/gpt-5.4-mini"
    And I send the message "S1 r5 x1"
    And I switch to the conversation "Chat Y"
    And I send the message "S1 r5 y1"
    Then the reply to "S1 r5 x1" came from "openai/gpt-5.4-mini"
    And the reply to "S1 r5 y1" came from "bdd-provider-b/claude-5-haiku"
    When I switch to the conversation "Chat X"
    And I clear the model override of this conversation
    And I confirm the change
    And I send the message "S1 r5 x2"
    And I switch to the conversation "Chat Y"
    And I send the message "S1 r5 y2"
    Then the reply to "S1 r5 x2" came from "bdd-provider-b/claude-5-haiku"
    And the reply to "S1 r5 y2" came from "bdd-provider-b/claude-5-haiku"
    When the operator changes the system default to "openai/gpt-5.4-mini"
    And I send the message "S1 r5 y3"
    Then the reply to "S1 r5 y3" came from "openai/gpt-5.4-mini"

  Scenario: A reply already being written finishes on the model it started with
    Given the stub supports "hold_content"
    And a conversation "Chat Y" with no model override
    And 5 seconds have passed since the system default was set
    When I send the message "S1 r6 hold" without waiting for the reply
    And the stub for "openai" is holding the reply to "S1 r6 hold"
    And the operator changes the system default to "bdd-provider-b/claude-5-haiku" from another page
    Then the reply to "S1 r6 hold" is still held and the stop control is still visible
    When the held reply is released
    Then the reply to "S1 r6 hold" finishes with content containing "bdd-reply-gpt-5.4-mini"
    And the reply to "S1 r6 hold" came from "openai/gpt-5.4-mini" with exactly one request
    And the answer's receipt names "openai/gpt-5.4-mini" and the origin "system default"
    And the answer's receipt shows the time the system default became "openai/gpt-5.4-mini"
    And the answer's receipt does not show the time the system default became "bdd-provider-b/claude-5-haiku"
    When I send the message "S1 r6 next"
    Then the reply to "S1 r6 next" came from "bdd-provider-b/claude-5-haiku"

  Scenario: A choice that cannot be served is refused and nothing is claimed
    Given a conversation "Chat X" with no model override
    When the operator tries to change the system default to "bdd-provider-off/off-model-1"
    Then the change is refused with a reason mentioning "disabled"
    And the registry default is still "openai"
    And the stored default provider is still "openai"
    When I try to set the model override of this conversation to "bdd-provider-b/claude-5-retired"
    Then the change is refused with a reason mentioning "not offered"
    And the "Chat X" conversation shows the effective model "openai/gpt-5.4-mini" from "system default"
    When I send the message "S1 r7 override refused"
    Then the reply to "S1 r7 override refused" came from "openai/gpt-5.4-mini"
    And no outgoing request used the model "off-model-1"
    And no outgoing request used the model "claude-5-retired"
    And no answer's receipt names "off-model-1"
    And no answer's receipt names "claude-5-retired"

  Scenario: An agent with its own model keeps it and is listed when the default changes
    Given an agent "BDD Agent Aurora" with the model "openai/gpt-5.4-mini" exists
    And an agent "BDD Agent Cirrus" with the extension model "bdd-provider-c/qwen-5-turbo" exists
    And an agent "BDD Agent Dawn" with no model exists
    When the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    Then the change result lists exactly "BDD Agent Aurora" and "BDD Agent Cirrus" as pinned
    And the change result's follow count equals the number of other agents
    When I switch to a fresh conversation using "BDD Agent Aurora"
    And I send the message "S1 r8 aurora"
    Then the reply to "S1 r8 aurora" came from "openai/gpt-5.4-mini"
    When I switch the active agent to "BDD Agent Cirrus"
    And I send the message "S1 r8 cirrus"
    Then the reply to "S1 r8 cirrus" came from "bdd-provider-c/qwen-5-turbo"
    When I switch the active agent to "BDD Agent Dawn"
    And I send the message "S1 r8 dawn"
    Then the reply to "S1 r8 dawn" came from "bdd-provider-b/claude-5-haiku"

  Scenario: Changing the default uses only the live route
    Given a fresh conversation using "Default Assistant"
    When the operator changes the system default to "bdd-provider-b/claude-5-haiku"
    Then the console's writes to the server were, in order, "PUT /api/uar/providers/bdd-provider-b" and "POST /api/uar/providers/bdd-provider-b/default"
    And the console wrote none of the settings "llm.default_provider", "llm.model" and "run_policy.global"

  Scenario: The receipt names the model the provider reported, not the one requested
    Given the stub supports "served_model"
    And the system default model is "bdd-provider-b/claude-5-haiku"
    And a conversation "Chat Y" with no model override
    When I send the message "S1 served"
    Then the stub of "bdd-provider-b" received one request for "S1 served" asking for the model "claude-5-haiku"
    And the answer's receipt names the serving model "claude-5-haiku-20261101" on "bdd-provider-b"
    And the answer's receipt does not name "claude-5-haiku" as the serving model
