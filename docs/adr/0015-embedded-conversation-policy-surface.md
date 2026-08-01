# 15. Embedded conversation-policy admin surface + effective-config resolver

Date: 2026-07-24

## Status

Accepted

## Context

ADR 0013 wired the embedded runtime to resolve the full Global → Agent →
Conversation → Turn precedence and exposed a settings/agent admin surface on the
SDK `Runtime`. The **Conversation** scope was already honored at resolution time
— `RunManager` reads `persistence.load_conversation_policy` when it builds the
resolution universe — and the persistence layer already had
`save/load/delete_conversation_policy`. But the SDK `Runtime` exposed **no way to
write or read** a conversation policy without standing up the HTTP service, and
no way to compute a conversation's effective configuration in-process.

Consequently first-party embedded hosts (mobile, in-process desktop) could offer
the Global and per-Agent model tiers (ADR 0013) but not the **per-conversation**
tier — the third scope of the Cherry-Studio-style model selection the product
wants. KnowMe's `control_plane::{conversation_policy, save_conversation_policy,
delete_conversation_policy, effective_config}` therefore returned
`not_on_embedded`.

## Decision

Expose the conversation scope on the embedded SDK `Runtime`, mirroring the
settings/agent surface from ADR 0013:

- `save_conversation_policy(conversation_id, RunPolicy) -> ConversationPolicyRecord`,
  `get_conversation_policy(conversation_id) -> Option<ConversationPolicyRecord>`,
  and `delete_conversation_policy(conversation_id)` — thin delegations to the
  persistence layer the runtime already owns.
- `effective_config(conversation_id) -> EffectiveConfig` — resolves the agent
  named by the stored conversation policy (or the default agent), resolves the
  effective run policy for that agent + conversation through the same
  transport-free core, and applies the model backfill (ADR 0014). It returns the
  resolved agent, the stored requested policy, and the effective policy — the
  same triple the service path's `GET /conversations/{id}/effective-config`
  returns.

The effective-config computation and the model backfill are factored into
`RunManager` methods (`effective_config`, `resolve_agent_or_default`,
`backfill_effective_model`), so the SDK facade stays a one-line delegation and
`start_run_with_policy` reuses the same backfill it introduced in ADR 0014.

## Consequences

- Embedded hosts can now read, write, and delete per-conversation model overrides
  and resolve a conversation's effective configuration with no HTTP service,
  completing the three-tier (global → agent → conversation) model selection on
  embedded parity with the service path.
- Precedence is unchanged: a conversation-scoped model overrides the agent and
  global scopes; deleting it reverts to the lower scopes and the registry-default
  backfill. Verified by `conversation_policy_round_trips_and_effective_config_
  reflects_the_override` (embedded lib test).
- No new persistence methods were added — the surface reuses the existing
  `save/load/delete_conversation_policy` the persistence trait already defined.
