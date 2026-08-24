# Session Configuration control inventory

| control | typed API field | persistence owner | runtime owner | disposition |
|---|---|---|---|---|
| Model Override | `AgentSessionConfig.model: string \| null` | `ConversationPolicyRecord.policy.model` through `POST /api/uar/sessions/{id}/agent-config` | `RunPolicy.model` and effective model-route resolution | Retained. Empty means inherit the agent default. |
| Tool Approval | `AgentSessionConfig.tool_approval: "auto" \| "ask" \| "deny" \| null` | `ConversationPolicyRecord.policy.tool_approval` through the same typed endpoint | `EffectiveRunPolicy.tool_approval` and the runtime tool-approval gate | Retained. `null` is presented as Agent default. |
| History Window | None | None | None | Removed. The former `context_strategy.history_window` key was ignored by Rust. |
| Inject Memory | None | None | None | Removed. The former `context_strategy.inject_memory` key was ignored by Rust. |
| Auto-capture | None | None | None | Removed. The former `context_strategy.auto_capture` key was ignored by Rust. |
| Memory Scope | None | None | None | Removed. The former `context_strategy.memory_scope` key was ignored by Rust. |
| Save Configuration | No independent field | Invokes the typed POST and replaces canonical `AgentSession` only after success | No independent runtime policy | Retained as the commit action; disabled until load succeeds and while saving. |

The POST body is created by `copyConfig` from the seven fields accepted by the
Rust `AgentSessionConfig`: `agent_id`, `model`, `tools`, `skills`,
`knowledge_bases`, `mcp_servers`, and `tool_approval`. Draft metadata is never
serialized. The retired `model_override` and `context_strategy` keys do not
exist in the typed transport.
