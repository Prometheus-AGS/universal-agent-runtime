# Product Surface Inventory

This inventory maps every route registered by `frontend/src/App.tsx` and every live admin section registered by `PAGE_MAP` to its owner, principal actions and APIs, current specification/test evidence, maturity, and remaining certification. “Missing” is intentional evidence of work still required; code presence is not a GA claim.

| Route or surface | Owner | Principal stable action(s) and API contract | Current spec / executable evidence | Maturity | Required certification |
|---|---|---|---|---|---|
| `/threads` chat | `features/chat`, chat stores/services | send/resume/cancel; `POST /v1/chat/completions`, `GET /api/uar/runs/{id}/stream`, `POST /api/uar/runs/{id}/cancel` | `chat-bdd-coverage`, `chat-scenario-coverage`; `frontend/e2e/chat-*.spec.ts`, Rust/Cucumber chat suites | Preview | `certify-agui-chat-flow` |
| `/threads` A2UI artifact block | `features/chat/components/a2ui-artifact-block` | render/submit artifact response; `POST /api/uar/runs/{id}/artifact-response` | `a2ui-live-testing`; existing live round-trip tests | Preview | `certify-a2ui-react-flow` |
| `/admin/runtime` cockpit | `admin/pages/runtime-console-page` | inspect live runs, provider health and activity; `/api/uar/providers/health` plus normalized runtime events | `runtime-console`, `runtime-console-ux`; `runtime-console-visual.spec.ts`, `runtime-event-replay.spec.ts` | Preview | `certify-runtime-console-governance` |
| `/admin/runs` | runtime console domain | inspect/replay/cancel runs; run detail/stream/cancel contracts | `runtime-console`, `runtime-event-replay-entity-sync`; replay E2E | Preview | `certify-runtime-console-governance` |
| `/admin/approvals` | runtime console domain | approve/deny tool calls; `POST /api/uar/runs/{id}/tool-approval` | `tool-approval-workflow`; backend tests, browser acceptance missing | Preview | `certify-runtime-console-governance` |
| `/admin/protocols` | runtime console domain | inspect AG-UI events, A2UI surfaces and model routes; `/api/uar/a2ui/schemas`, `/api/uar/resolve-model` | runtime-console specs; runtime visual/replay E2E | Preview | `certify-agui-chat-flow`, `certify-a2ui-react-flow` |
| `/admin/providers` | provider entity domain | list/configure/default/delete; `/api/catalog`, `/api/uar/providers[/{id}[/default]]` | `provider-diagnostic-status`; `admin-providers.spec.ts` | Preview | `certify-provider-model-settings-flow` |
| `/admin/credentials` | credential store/service | list/upsert/delete masked credentials; `/api/uar/credentials[/{provider}]` | no focused OpenSpec baseline; `credentials-page.test.ts` is unit-only | Preview | `certify-remaining-admin-surfaces` |
| `/admin/models` | model/provider domain | browse and resolve; `/api/models`, `/api/uar/resolve-model` | `openai-models-endpoint`; browser acceptance missing | Preview | `certify-provider-model-settings-flow` |
| `/admin/skills` | skill entity domain | list/create/update/toggle/import; `/api/skills[/{id}[/toggle]]`, `/api/uar/skills/import` | `skill-hot-reload`; `admin-skills.spec.ts`, utility unit tests | Preview | `certify-remaining-admin-surfaces` |
| `/admin/agents` | agent entity domain | list/create/update/delete/status; `/api/agents[/{id}]` | `agent-status-ui`; `admin-agents.spec.ts`, agent selection E2E | Preview | `certify-remaining-admin-surfaces` |
| `/admin/tools` | tool entity domain | discover/inspect tools; `/api/tools` | `tool-approval-workflow` covers execution governance; `admin-tools.spec.ts` | Preview | `certify-remaining-admin-surfaces` |
| `/admin/auth` | auth store/service | list/create/revoke API keys; `/api/auth/keys[/{id}]` | `auth-key-management`; comprehensive backend tests, browser acceptance missing | Preview | `certify-remaining-admin-surfaces` |
| `/admin/knowledge` | knowledge entity domain | list/create/delete KBs, upload/list/delete/search documents; `/api/knowledge...`, upload API | local embedding spec; `admin-knowledge.spec.ts`, RAG integration test | Preview | `certify-knowledge-rag-flow` |
| `/admin/memory` | memory entity domain | list/stats/delete/clear; `/api/admin/memories...` | no focused product-surface spec; browser acceptance missing | Preview | `certify-remaining-admin-surfaces` |
| `/admin/compiler` | compiler domain | list/create compiler sessions; `/api/compiler/sessions` | no focused product-surface spec; browser acceptance missing | Preview | `certify-remaining-admin-surfaces` |
| `/admin/settings` | settings domain | load/update namespaces and types; `/api/uar/settings...` | frontend validation/config specs are partial; `use-settings.test.tsx` | Preview | `certify-provider-model-settings-flow` |
| `/admin/a2ui-testing` | A2UI feature | list schemas and trigger a test artifact; `/api/uar/a2ui/schemas`, `/api/uar/runs/{id}/a2ui/test-trigger` | `a2ui-testing-ui`, `a2ui-live-testing`; live round-trip tests | Developer preview | `certify-a2ui-react-flow` |
| `/admin/mcp-health` | MCP domain | inspect server health; `/api/uar/mcp/health` | `mcp-health-dashboard`; browser acceptance missing | Preview | `certify-remaining-admin-surfaces` |
| `/admin/cost` | runtime cost entities | inspect usage/budget alerts; runtime event/entity contracts | no focused product-surface spec; browser acceptance missing | Preview | `certify-remaining-admin-surfaces` |
| `/about` | `pages/about-page` | render build/product metadata; static client contract | no focused product-surface spec or acceptance test | Preview | `reconcile-product-documentation` |

## Required action evidence

Every stable action must eventually have:

1. an owning store/domain action and typed service;
2. success, failure, empty and authorization behavior where applicable;
3. realtime or persistence reconciliation when state is durable;
4. an OpenSpec requirement and executable test identifier;
5. a support-matrix row linked to release evidence.
