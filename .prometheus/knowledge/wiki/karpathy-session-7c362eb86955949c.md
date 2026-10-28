---
type: SessionRecord
id: karpathy-session-7c362eb86955949c
title: Karpathy session 7c362eb86955
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T09:06:26.204586+00:00
created_at: 2026-10-28T09:06:26.204586+00:00
updated_at: 2026-10-28T09:06:26.204586+00:00
revision: 0
---

## Delta

The closest matches are chat platforms that let bots post interactive UI. They have the same parts as UAR: an agent sends UI as data, users click, the bot answers, and clients reconnect. A few other systems show useful variations.

I'm describing these from memory and haven't re-verified them in this session. The analysis will cite sources for each one I rely on.

## 1. Slack Block Kit: the UI is a message in the conversation

- **Rendering.** A bot posts a message made of "blocks": buttons, inputs, selects. The message is stored in the channel's history like any other message, identified by its timestamp (`ts`).
- **Clicks.** A click sends the bot a payload with the message id and **the current values of every input** (`state.values`).
- **Updates.** The bot changes the UI by replacing the **whole message** (`chat.update` with the channel and `ts`), or by replying through a `response_url`.
- **Reconnects.** The client re-syncs channel history, like any message. Slack keeps no per-client record of the UI.

What this shows: the UI belongs to the conversation, not to the bot's request. Updates replace the whole object, so there's no fine-grained diffing. The user's input travels with the click instead of being kept as server state.

## 2. Microsoft Teams Adaptive Cards: the same idea, plus refresh on view

- **Rendering and updates.** A card is a JSON attachment on a message in the conversation. The bot updates it by replacing that activity by id.
- **Refresh on view.** With Universal Actions, a card can declare a `refresh` action. When a user opens the conversation, the client asks the bot for the current card, and can ask for a per-user version.

What this shows: recovery can be **pull on view** ("give me the current card"), not replaying what was missed.

## 3. Vercel AI SDK (RSC) and OpenAI's Apps SDK: persist the model, rebuild the UI

- **AI SDK RSC (experimental).** It separates a serializable **AI state**, saved server-side with the chat, from a **UI state** that exists only on the client. On reload, the UI is rebuilt from the saved AI state (`onGetUIState`).
- **Apps SDK.** A widget in ChatGPT can save `widgetState`. The host stores it with that message instance, so the widget comes back as the user left it.

What this shows: two standard answers for unsent input. Rebuild the UI from saved state, or let the host save a small per-message widget state. That is the drafts row from my last answer.

## 4. Phoenix LiveView: the server owns the UI and rebuilds it on reconnect

- **The server owns the UI.** Each connected client gets a server process that holds the UI state and pushes rendered diffs.
- **Reconnect.** On reconnect the process is gone. The server mounts again from durable data and sends a full render.
- **Form recovery.** The client automatically resubmits the values in open forms, so typed input survives.

What this shows: even a server that tracks each connected client throws that tracking away on disconnect. Recovery comes from durable data plus input the client sends back, not from remembering the client.

## 5. Linear and Figma: sync engines with one data model

- **Linear** (from its public talks). Each client keeps a local database. The server numbers every change with a sync id that only goes up. On reconnect the client asks for "changes since my last sync id". If it has fallen too far behind, it downloads everything again.
- **Figma** (from its multiplayer engineering post). The server holds each document. A client downloads the whole document on connect, then exchanges changes. Conflicts resolve per property, with the last write winning. Edits made offline are sent on reconnect.

What this shows: a version cursor with a full-download fallback. Fine-grained diffs only work here because **one team owns one data model on both sides**. UAR's attempt broke down on exactly that, because it was copying someone else's reducer.

## What they have in common

1. **The UI lives with the durable conversation or docu

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T09:06:22.900524Z
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
