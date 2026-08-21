# Product screen validation matrix

Profile: local Chromium; Vite frontend; UAR `server-full`; deterministic stub
LLM; fresh embedded SurrealKV; browser PGlite. Results transfer to no other
browser, runtime profile, provider, deployment, or platform.

Each row requires its own passing Cucumber scenario and Playwright video. A
route or heading check alone does not satisfy a mutating primary function.

| # | Screen | Purpose | Primary function under test | Required observation | Scenario | Result | Video |
|---:|---|---|---|---|---|---|---|
| 1 | `/threads` | Converse with a selected agent | Send a deterministic prompt through the visible composer | Selected agent returns the exact stubbed answer and the transcript persists after reload | `Chat returns and persists a deterministic answer` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-3f0bb-ists-a-deterministic-answer.mp4` |
| 2 | `/about` | Show product identity and runtime availability | Load live runtime health metadata | Product identity and the exact live health state render | `About reports product and runtime identity` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-593ae-roduct-and-runtime-identity.mp4` |
| 3 | `/admin/runtime` | Operate the live runtime cockpit | Receive a known runtime event | Cockpit state changes without page reload | `Runtime cockpit consumes a live event` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-d9179-ckpit-consumes-a-live-event.mp4` |
| 4 | `/admin/runs` | Inspect run traces | Open a replayed run | Run detail shows the known step/artifact state | `Runs opens replayed trace detail` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-731ba-opens-replayed-trace-detail.mp4` |
| 5 | `/admin/approvals` | Govern tool approvals | Deny a pending approval | Approval becomes denied in the live surface | `Approvals denies a pending tool call` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-090bd--denies-a-pending-tool-call.mp4` |
| 6 | `/admin/protocols` | Inspect protocol and routing events | Receive known AG-UI/A2UI/routing events | Each protocol family is visible without reload | `Protocols shows live event families` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-2448e-s-shows-live-event-families.mp4` |
| 7 | `/admin/providers` | Configure model providers | Change the default provider, observe the new route, then restore the original | The alternate provider becomes default and the original provider is restored | `Providers changes and restores the default route` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-eafbc--restores-the-default-route.mp4` |
| 8 | `/admin/credentials` | Manage user-scoped provider credentials | Store then remove a credential as a verified subject | Masked credential appears, then is absent | `Credentials stores and removes a user secret` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-b0d9b-s-and-removes-a-user-secret.mp4` |
| 9 | `/admin/models` | Browse resolved models | Filter the live model catalog and select a result for comparison | Known stub model is shown and the Compare selection becomes active | `Models filters the live catalog` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-25508-ls-filters-the-live-catalog.mp4` |
| 10 | `/admin/skills` | Govern runtime skills | Create, disable, and re-enable a skill | The skill appears and its enabled state changes live | `Skills completes the lifecycle` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-00e52-lls-completes-the-lifecycle.mp4` |
| 11 | `/admin/agents` | Manage runtime agents | Create and select a configured agent | Agent appears and can be selected in chat | `Agents creates a selectable agent` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-a4ef3--creates-a-selectable-agent.mp4` |
| 12 | `/admin/tools` | Inspect governed tools | Find and open the native echo tool | Built-in source metadata and the schema/test surface render | `Tools finds the native echo tool` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-8181b--finds-the-native-echo-tool.mp4` |
| 13 | `/admin/auth` | Manage API keys | Mint then revoke a named API key | Raw key is shown once and the key disappears after revoke | `Auth mints and revokes an API key` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-36e30-ints-and-revokes-an-API-key.mp4` |
| 14 | `/admin/knowledge` | Manage searchable knowledge | Create a KB, upload a document, and search it | Indexed document returns the known content | `Knowledge indexes and searches a document` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-78cff-xes-and-searches-a-document.mp4` |
| 15 | `/admin/memory` | Inspect and filter memory | Query live memory statistics and filter by verified user | Stats resolve and the filter is applied without an error state | `Memory filters verified-user state` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-a6bd5-filters-verified-user-state.mp4` |
| 16 | `/admin/compiler` | Manage compiler sessions | Create an ephemeral compiler session | New session is visible with a concrete identifier/status | `Compiler creates a session` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-91d08--Compiler-creates-a-session.mp4` |
| 17 | `/admin/settings` | Edit runtime/user settings | Change one reversible setting and save | Authoritative value survives reload, then is restored | `Settings saves and restores a value` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-1fab0--saves-and-restores-a-value.mp4` |
| 18 | `/admin/a2ui-testing` | Exercise developer-preview A2UI | Preview the deterministic test artifact | A2UI artifact is visible in the development-only surface | `A2UI testing previews an artifact surface` | Pass (development profile only) | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-42414-reviews-an-artifact-surface.mp4` |
| 19 | `/admin/mcp-health` | Diagnose MCP servers | Refresh live MCP health | Server status and tool count resolve from the API | `MCP health refreshes server status` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-c0957-lth-refreshes-server-status.mp4` |
| 20 | `/admin/cost` | Inspect priced-run and budget state | Receive known priced-run state | Exact total spend, priced-run count, and model breakdown render | `Cost displays known usage` | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-product-screen-va-77bac-n-Cost-displays-known-usage.mp4` |

## Cross-screen scenarios

| Requirement | Positive observation | Required failing control | Result | Video |
|---|---|---|---|---|
| Orchestrator/default-agent answers | Exact deterministic answers and exact `[rust-reviewer]` attribution stream through the visible chat UI | Exact assertions reject any additional or alternative text | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-cross-screen-secu-f688e-rn-exact-attributed-answers.mp4` |
| Skill and RAG evidence | Skill activation and KB citation/source render in the transcript | No-KB scenario renders no citation markers | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-chat-skill-activa-3b818--activates-mid-conversation.mp4`; `docs/certifications/product-screens/f8e203b6/videos/features-rag-citation.feat-52224--knowledge-base-is-attached.mp4` |
| Memory levels | Global, agent, and user memory rows are written and reread with explicit scope assertions | A second verified subject in the same tenant cannot read the first subject's user-owned state | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-cross-screen-secu-3bf82--user-state-remains-private.mp4` |
| JWT enforcement | Verified JWT receives 200 from the credential route | Anonymous request receives 401 | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-cross-screen-secu-6b76b--and-anonymous-access-fails.mp4` |
| Two-user isolation | Owning subject reads its session/memory/KB state | A different verified subject in the same tenant receives 404/no matching row | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-cross-screen-secu-3bf82--user-state-remains-private.mp4` |
| Offline behavior | Browser offline event exposes the shipped banner | Returning online removes it | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-local-first-resil-881ee-ble-and-clears-on-reconnect.mp4` |
| PGlite persistence | Created thread/message survives a browser reload | A fresh browser context does not inherit it | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-local-first-resil-f9399-only-in-its-browser-context.mp4` |
| SSE resynchronization | The Knowledge screen applies an embedded entity change and the server-updated name after reconnect | Reconnect leaves exactly one recovered knowledge base | Pass | `docs/certifications/product-screens/f8e203b6/videos/features-local-first-resil-d7113-t-duplicating-runtime-state.mp4` |

## Observed defects repaired

The suite first exposed three supported-screen defects: Skills ignored live
graph rows without a list index, approval-required events never reached the
chat/runtime graph, and Knowledge cards nested a delete button inside their
selection button. The final candidate repairs all three before reporting their
screen scenarios as passing. The uncomfortable consequence is that the original
32-video bundle is superseded and cannot be used as evidence for this candidate.
