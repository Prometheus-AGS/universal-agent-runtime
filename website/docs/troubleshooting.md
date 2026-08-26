---
sidebar_position: 7
title: Troubleshooting
description: Route operational symptoms to the current bounded guides.
---

# Troubleshooting

Start in [Operate the Runtime Console](/docs/operations/runtime-console) to distinguish live server state from the browser projection. Then follow the boundary that matches the symptom:

| Symptom | Current guide |
|---|---|
| 401, JWKS failure, or provider conflict | [Authenticate Requests](/docs/security/authentication) |
| Provider key unavailable or credential API returns 503 | [Manage Provider Credentials](/docs/security/credentials) |
| Policy denial or unexpected permit behavior | [Apply Governance Policies](/docs/governance/overview) |
| Tool call waits or rejects | [Resolve Tool Approvals](/docs/governance/approvals) |
| Stream disconnect, missed live update, or stale browser state | [Understand Realtime State](/docs/operations/realtime) |
| Empty metrics, readiness failure, or missing traces | [Observe the Runtime](/docs/operations/observability) |
| Prompt caching setting fails to load, or cache reads stay at zero | [Configure Prompt Caching](/docs/providers/prompt-caching) |
| Cost is absent or differs from provider totals | [Interpret Cost and Budgets](/docs/operations/cost) |
| Datastore lock, forced exit, backup, or restore question | [Recover and Shut Down](/docs/operations/recovery-and-shutdown) |

Record the active profile, source revision, exact request or command, status, and relevant redacted logs. Do not include credentials, private keys, or raw user/session payloads in a public issue.
