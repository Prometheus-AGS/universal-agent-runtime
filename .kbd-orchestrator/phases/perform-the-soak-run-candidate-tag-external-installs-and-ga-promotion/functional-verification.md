# UAR 1.0 Functional Closeout Verification

**Date:** 2026-08-22  
**Profile:** `server-full` only  
**Provider boundary:** packaged UAR server with OpenAI-compatible proxy at `http://localhost:8181` and Kimi for Coding  
**Source checkout:** `uar-1-0-readiness` worktree on `codex/complete-kbd-79`

No aggregate percentage or cross-profile verdict is made.

| Requirement | Surface and action | Observed result | Stated limit |
| --- | --- | --- | --- |
| OpenAI proxy inference | API: `POST /v1/chat/completions` through packaged UAR using `openai/gpt-5.4-mini`; provider base URL `http://localhost:8181/v1` | HTTP 200; `UAR-PROXY-REAL-8181` | Proves one genuine response through this local `server-full` provider path. |
| OpenAI proxy inference | UI: Chat asked `23 * 19` with OpenAI selected | UI returned `437` | Proves the shipped UI completed one genuine response through this local provider path. |
| Skill activation with inference | API: created `emberquartz-functional-skill`, then streamed a prompt containing its activation keyword | Activation selected `skill_service.keyword`; model returned `SKILL-ACTIVATED-EMBER` | Proves this skill activated and influenced one genuine inference request. |
| Skill activation with inference | UI: enabled/selected skills and sent the Emberquartz activation prompt | UI displayed `Emberquartz Functional Skill` activation and returned `SKILL-ACTIVATED-EMBER` | Proves activation through the shipped UI for this skill and request. |
| Knowledge grounding | API: created `Firefly Functional KB`, uploaded a fact containing `Cobalt Heron 7319`, indexed it, and asked the model for the fact with that KB selected | Search score `0.8674219`; inference returned `Cobalt Heron 7319` | Proves one indexed fact was retrieved and used in one genuine inference request. |
| Knowledge grounding | UI: created `UI Verification KB`, uploaded `ui-knowledge-fact.txt` containing `Silver Mango 4826`, selected the KB, and asked for the fact | UI returned `Silver Mango 4826` and displayed `ui-knowledge-fact.txt` as the source | Proves UI creation, upload, selection, retrieval, and grounded inference for this KB. |
| Kimi k3 configuration and inference | API: configured `kimi-for-coding` model `k3` without sending a key in the request, relying on the catalog-declared environment credential | Response reported `credential_configured: true`; HTTP 200 inference returned `KIMI-K3-LIVE-UAR` | Proves environment-backed Kimi credentials and one genuine k3 inference through local `server-full`. |
| Kimi k3 configuration and inference | UI: edited Kimi provider configuration, created `UI Kimi K3 Agent`, selected it in Chat, and sent a fresh prompt | Effective run policy reported `agent_id: ui-kimi-k3-agent`, `provider_id: kimi-for-coding`, `model_id: k3`; model `k3` returned `UI-KIMI-K3-LIVE` | Proves the shipped UI routed to Kimi k3 rather than the global OpenAI default. |
| Basic agent creation and inference | API: created `api-basic-agent` and sent an inference request through UAR | Returned `AGENT-BASIC-LIVE` | Proves API creation and one genuine inference request using the created agent. |
| Basic agent creation and inference | UI: created `UI Basic Agent` with `openai/gpt-5.4-mini`, selected it in Chat, and sent a fresh prompt | Effective run policy reported `agent_id: ui-basic-agent`, `provider_id: openai`, `model_id: gpt-5.4-mini`; the model returned a genuine response | The model refused to reveal its hidden prompt, which is valid behavior; agent/model routing and response completion are the bounded claim. |

## Code-completion corrections made before verification

- `src/llm/registry.rs` resolves catalog-declared provider credential environment variables.
- `frontend/src/features/providers/model/providers-store.ts` updates existing providers instead of treating HTTP 409 as success.
- `src/uar/runtime/native_skill.rs` maps dotted internal skill IDs to provider-compatible tool names and resolves aliases on lookup.
- `frontend/src/components/assistant-ui/enhanced-thread.tsx` renders grouped leaf parts without returning invalid child values.
- `frontend/src/features/chat/chat-thread-view.tsx` creates the chat runtime inside the selected-agent and memory providers, so requests carry the UI-selected agent and model.
- The shipped static frontend bundle was rebuilt with `pnpm build`; Vite completed successfully after transforming 8,086 modules.

## Explicit exclusions

No unit suite, synthetic provider, recorded provider, soak, supply-chain certification,
release-candidate certification, tag, GA publication, GitHub Actions job, push, or PR
was run for this closeout. The four former release-tail changes are cancelled and
must not be inferred as passing from these functional results.
