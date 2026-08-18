# Product screen validation matrix

Profile: local Chromium; Vite frontend; UAR `server-full`; deterministic stub
LLM; fresh embedded SurrealKV; browser PGlite. Results transfer to no other
browser, runtime profile, provider, deployment, or platform.

Each row requires its own passing Cucumber scenario and Playwright video. A
route or heading check alone does not satisfy a mutating primary function.

| # | Screen | Purpose | Primary function under test | Required observation | Scenario | Result | Video |
|---:|---|---|---|---|---|---|---|
| 1 | `/threads` | Converse with a selected agent | Send a deterministic prompt through the visible composer | Selected agent returns the exact stubbed answer and the transcript persists after reload | `Chat returns and persists a deterministic answer` | Pending | Pending |
| 2 | `/about` | Show build and product identity | Load runtime health/build metadata | Product identity and live health/version state render | `About reports product and runtime identity` | Pending | Pending |
| 3 | `/admin/runtime` | Operate the live runtime cockpit | Receive a known runtime event | Cockpit state changes without page reload | `Runtime cockpit consumes a live event` | Pending | Pending |
| 4 | `/admin/runs` | Inspect run traces | Open a replayed run | Run detail shows the known step/artifact state | `Runs opens replayed trace detail` | Pending | Pending |
| 5 | `/admin/approvals` | Govern tool approvals | Deny a pending approval | Approval becomes denied in the live surface | `Approvals denies a pending tool call` | Pending | Pending |
| 6 | `/admin/protocols` | Inspect protocol and routing events | Receive known AG-UI/A2UI/routing events | Each protocol family is visible without reload | `Protocols shows live event families` | Pending | Pending |
| 7 | `/admin/providers` | Configure model providers | Register and select a deterministic provider | Provider becomes configured/default and its model route is visible | `Providers configures the stub provider` | Pending | Pending |
| 8 | `/admin/credentials` | Manage user-scoped provider credentials | Store then remove a credential as a verified subject | Masked credential appears, then is absent | `Credentials stores and removes a user secret` | Pending | Pending |
| 9 | `/admin/models` | Browse resolved models | Filter the live model catalog | Known stub model is shown and selectable | `Models filters the live catalog` | Pending | Pending |
| 10 | `/admin/skills` | Govern runtime skills | Create, disable, and re-enable a skill | The skill appears and its enabled state changes live | `Skills completes the lifecycle` | Pending | Pending |
| 11 | `/admin/agents` | Manage runtime agents | Create and select a configured agent | Agent appears with its configured model and can be selected in chat | `Agents creates a selectable agent` | Pending | Pending |
| 12 | `/admin/tools` | Inspect governed tools | Find the native echo tool | Tool metadata and governed-execution status render | `Tools finds the native echo tool` | Pending | Pending |
| 13 | `/admin/auth` | Manage API keys | Mint then revoke a named API key | Raw key is shown once and the key disappears after revoke | `Auth mints and revokes an API key` | Pending | Pending |
| 14 | `/admin/knowledge` | Manage searchable knowledge | Create a KB, upload a document, and search it | Indexed document returns the known content and citation metadata | `Knowledge indexes and searches a document` | Pending | Pending |
| 15 | `/admin/memory` | Inspect and filter memory | Query live memory statistics and filter by verified user | Stats resolve and the filter is applied without an error state | `Memory filters verified-user state` | Pending | Pending |
| 16 | `/admin/compiler` | Manage compiler sessions | Create an ephemeral compiler session | New session is visible with a concrete identifier/status | `Compiler creates a session` | Pending | Pending |
| 17 | `/admin/settings` | Edit runtime/user settings | Change one reversible setting and save | Authoritative value survives reload, then is restored | `Settings saves and restores a value` | Pending | Pending |
| 18 | `/admin/a2ui-testing` | Exercise developer-preview A2UI | Trigger the test artifact | A2UI artifact becomes visible; result is labelled development-only | `A2UI testing triggers an artifact` | Pending | Pending |
| 19 | `/admin/mcp-health` | Diagnose MCP servers | Refresh live MCP health | Server status and tool count resolve from the API | `MCP health refreshes server status` | Pending | Pending |
| 20 | `/admin/cost` | Inspect usage and budget state | Receive known usage/cost state | Token/cost values render in the dashboard | `Cost displays known usage` | Pending | Pending |

## Cross-screen scenarios

| Requirement | Positive observation | Required failing control | Result | Video |
|---|---|---|---|---|
| Orchestrator/default-agent answers | Exact deterministic answers stream through the visible chat UI | Wrong expected answer makes the assertion fail | Pending | Pending |
| Skill and RAG evidence | Skill activation and KB citation/source render in the transcript | Unbound skill/no-KB cases show neither indicator | Pending | Pending |
| Memory levels | User, agent, and conversation memory evidence is distinguishable | A second verified subject cannot read the first subject's state | Pending | Pending |
| JWT enforcement | Verified JWT reaches a protected browser-owned request | Anonymous request is rejected | Pending | Pending |
| Two-user isolation | Owning subject reads its session/memory/KB state | Different verified subject is denied or receives no row | Pending | Pending |
| Offline behavior | Browser offline event exposes the shipped banner | Returning online removes it | Pending | Pending |
| PGlite persistence | Created thread/message survives a browser reload | A fresh browser context does not inherit it | Pending | Pending |
| SSE resynchronization | Known event appears live and restored state appears after reconnect | Stale/unknown event does not corrupt the surface | Pending | Pending |

## Uncomfortable fact

This matrix is a plan until every result and video cell is replaced by an
observed artifact. Existing route-smoke tests and the prior six-video chat
bundle reduce setup cost but do not satisfy the 20-screen requirement by
themselves.
