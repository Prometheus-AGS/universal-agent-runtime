---
type: SessionRecord
id: karpathy-session-ef93d529d0f4bb54
title: Karpathy session ef93d529d0f4
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T15:59:45.595566+00:00
created_at: 2026-11-19T15:59:45.595566+00:00
updated_at: 2026-11-19T15:59:45.595566+00:00
revision: 0
---

## Delta

Completed adversarial-review — BLOCK (3/4/1), with the cap reached, so the last round's corrections went in un-reviewed.

Completed kbd-assess — uar-uiux-architecture-2026-11 (step 0 of 0)

The assessment is written to [assessment.md](.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/assessment.md). Every one of the nine goals is partial or not met. Nothing was built, run or tested; this comes from reading the source.

**Main findings**
- **Config sync (goal 3):**
  - Editing the YAML while the server runs changes nothing. The file watcher reloads it, but the server reads its config only once, at startup (`server.rs:490`).
  - Saving invalid YAML while the server runs is worse: nothing reports the problem, and the next restart fails to boot.
  - Many settings the UI lets you edit are saved to the database but never read by anything.
  - If one stored provider row is malformed, the server still starts but silently drops every provider stored in the database.
- **Your config decision was undone without anyone saying so.** In June you decided YAML wins on every boot, and that was implemented in `c41da57e`. On 2026-07-20 an unrelated feature commit, `7a6aa305`, quietly reverted it to database-wins. That July change is still recorded as an open OpenSpec change.
- **Model switching (goal 4):** agent and conversation changes take effect on the next turn.
  - The system level has three competing "default model" settings, and none of them is both reachable in the UI and actually used.
  - Teams lock their models when they start, and an edit to a model a running team uses blocks that team's next task.
- **Pickers and settings (goals 6 and 7):**
  - About 11 model or provider fields are still typed by hand, including the embedding model actually in use.
  - One generic settings form shows plain text, comma-separated lists and raw JSON boxes across 13 settings sections.
  - The RAG embedding fields are rejected by the backend, and saving them also throws away any valid RAG edits made at the same time.
- **UI clutter (goals 5 and 9):** 8 admin pages can't be reached from anywhere in the app. Six settings areas duplicate pages elsewhere, and Settings has 30 sections you can't link to directly.
- **PGlite and web workers (goals 1 and 8):**
  - The browser database runs on the main thread, and the app has no web workers at all.
  - On a fresh install, the first session's message writes probably fail because the database starts from an out-of-date v3 seed. A reload fixes it.
  - Models are stored twice in the client state under two different IDs.
- **Impeccable (goal 2):** the three installed Impeccable version numbers disagree (4.9.2, 0.1.19 and 0.1.17). `frontend/PRODUCT.md` says this work "does not redesign the application", which contradicts this phase, and there is no `DESIGN.md`.
- **Open changes:** 259 OpenSpec changes are still unarchived, and many overlap this phase. Some of their task lists describe code that is no longer there.

**How it was reviewed, and what I couldn't do**
- **External review blocked:** the external review model was blocked by the permission classifier, so I used the documented fallback: a fresh reviewer agent in this session that saw only the document. That reviewer is the same model family as me, so this was not a cross-model review.
- **Corrections made:** across two rounds the reviewer found 5 critical problems in my draft, and I corrected all of them. One round-1 correction I applied without checking turned out to be wrong, and the record says so.
- **Not re-reviewed:** two rounds is the cap, so the round-2 corrections were not reviewed again. They are listed under "Unresolved review findings".
- **Missing runtime records:** the KBD runtime still has no record of this phase, so it couldn't load the phase's method settings. The `prometheus kbd stage` start and finish events weren't recorded either. Registering the phase permanently adds it to the runtime's append-only log, so I left that for you (questio

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T15:54:23.143201Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
