---
type: SessionRecord
id: karpathy-session-c7316c1ce0058c42
title: Karpathy session c7316c1ce005
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T09:09:37.789922+00:00
created_at: 2026-10-28T09:09:37.789922+00:00
updated_at: 2026-10-28T09:09:37.789922+00:00
revision: 0
---

## Delta

This changes the requirements more than the transport. A chat card is a short-lived reply. A full admin dashboard is a long-lived application screen, and that turns UAR into a **server-driven UI runtime**, which is a bigger thing to build.

## First, two possible readings

1. **One custom A2UI component that *is* the dashboard.** It becomes a black box, much like an embedded mini web app. The agent can't compose it or read it. Its internals would have to call APIs directly, so interaction would no longer go through AG-UI alone. I don't recommend this.
2. **A dashboard *built from* A2UI components.** Navigation, tables, charts, filters and forms are each declared components bound to a data model. Every meaningful interaction is an AG-UI exchange. That's what you described, and it's what the rest of this answer covers.

## What changes compared with a chat card

| | Chat card | Admin dashboard |
|---|---|---|
| Lifetime | One conversation turn | An application session, open for hours, in several tabs |
| Who handles clicks | The agent (a model run) | Mostly **deterministic handlers**. A model run per filter click is too slow and too expensive |
| Update rate | A few messages | Frequent small data updates (live metrics, table pages) |
| Data size | Small | Thousands of rows, so the server must page |
| Security | Validate what the agent emits | Also **authorize every user action** against what that user was shown |
| Identity | Optional | Required: per-user views and roles |
| Navigation | None | Pages, deep links, back button |

## Architecture sketch

**1. A server-owned surface store.** This is option B.
- Each surface's current components and data model are stored durably, with a version number.
- Each surface belongs to an application session owned by a user, not to one run.
- This is required, not just preferred. Without it the server can't check that an incoming action was actually offered to this user on this version of the screen. Option C, where only clients hold state, can't do that check.

**2. Two kinds of update, with different rules.**
- **Structure (components)** changes rarely: replace components whole, by id.
- **Data** changes often: path updates under a **narrow, UAR-defined rule** (option E). Paths only into containers that exist, no gaps, a depth limit, a size limit.
- The rule is narrow enough that the Rust server and `web_core` in the browser give the same result without one copying the other. That avoids the dual-reducer problem that sank change 1, while keeping updates small for live dashboards.

**3. Surface controllers.**
- Each surface type registers deterministic action handlers on the server, like a route handler in a web framework. "Filter users", "next page" and "disable account" run code, not a model.
- The agent is optional. It can create or adapt surfaces, or handle natural-language requests ("show me inactive admins"), but it isn't in the path of routine clicks.

**4. AG-UI as the only channel.**
- **Client to server.** An action is an AG-UI request on the session's thread, carrying the surface id, the action name, the input values and the surface version the user saw.
- **Server to client.** The response stream carries the resulting A2UI updates and the new version.
- **Pushed updates** (live metrics, changes from another tab or another admin) need a long-lived event stream per thread, not one per run. AG-UI is organized around runs, so this part needs checking against the upstream spec, which the research report is covering.
- **Resume** works by version cursor, as discussed: send changes since version N, or a snapshot if N is too old.

**5. Local interaction stays local.**
- Typing in a filter box, switching tabs, hovering and client-side validation update the client's data model through A2UI input bindings. They don't touch the network.
- Only deliberate actions go to the server, carrying the values they need, as Slack sends `state.values` with a click.

**6. The catalog becomes the produc

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T09:09:34.729554Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/progress.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/tasks.md
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/hooks-status.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/hooks.log.jsonl
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/progress.json
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/tasks.md
- .kbd-orchestrator/position-reminder.txt
- .kbd-orchestrator/position.json
- .prometheus/decisions.md
- .prometheus/session-log.md
- .prometheus/verification/macos-behavior-ledger.md
- openspec/changes/agui-runs-stream-fidelity/proposal.md
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/agui-a2ui-projection-and-custom-components/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agui-a2ui-projection-and-custom-components/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S6/attempt-1/universal-agent-runtime
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S7/attempt-1/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S8/attempt-2/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-004/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-005/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-006/artifacts/
- .prometheus/verification/browser-agent-selection/c53-attempt-001-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-002-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-003-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-004-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-005-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-001-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-002-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-003-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-004-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-005-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-006-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- docs/validation/
- history.txt
- openspec/changes/a2ui-surface-state-patches/
- openspec/changes/agui-a2ui-projection-and-custom-components/
- sdks/typescript/.acceptance-artifacts/
