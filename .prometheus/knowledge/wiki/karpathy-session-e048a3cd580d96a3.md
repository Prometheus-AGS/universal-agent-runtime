---
type: SessionRecord
id: karpathy-session-e048a3cd580d96a3
title: Karpathy session e048a3cd580d
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T19:44:25.414843+00:00
created_at: 2026-11-19T19:44:25.414843+00:00
updated_at: 2026-11-19T19:44:25.414843+00:00
revision: 0
---

## Delta

Analyze is finished, but Spec is blocked on one decision from you. The analysis is in [analysis.md](.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/analysis.md), with the machine-readable list in `library-candidates.json`. The stage is recorded as complete in the KBD runtime. Nothing was built or tested, and nothing is committed.

**What it found that changes the assessment**
- **Remote realtime push already exists.** `/api/live` works on remote Surreal and Postgres. The real gaps are specific topics:
  - Conversation policy changes are never published, on any backend.
  - Provider changes go out on the wrong topic on Surreal.
  - Nothing ever publishes to the `models` topic.
  - Team updates aren't public.
- **The system-default model only takes effect at restart through Settings.** Editing `llm.default_provider` changes nothing until restart, yet the API reports the edit as effective. Only `POST /api/providers/{id}/default` switches the default live. The system picker has to use that route.
- **Running teams are also blocked by any binding revision change** (`resolution.rs:39-48`), not just by route changes. Your "next task boundary" decision needs that check relaxed, but only for model changes.
- **The YAML problem is at startup, not on hot reload.** Reload already rejects a bad file. A bad file only stops the next start.
- **None of the 15 settings namespaces is read from the database.** 11 are honored from YAML at startup and 4 aren't read at all. Removing them from the UI is safe.

**Libraries:** everything needed is already installed except the worker setup:
- **Pickers:** Base UI combobox with list virtualization.
- **Config:** `ConfigManager`.
- **Realtime:** the existing Surreal and Postgres change feed.
- **Graph persistence:** PEM 5.0.1. Its `persistNow()` gives the save-deadline we need without a PEM release.
- **Workers:** PGlite's built-in worker mode, after moving the seed and migrations into it.

I rejected RJSF and comlink. There are 23 build items, so Plan will probably need to split this phase into child phases.

**Review:** two fresh critics reviewed it, both the same model family as me, because the cross-model judge was blocked earlier. Round 1's critical findings were real and I fixed them. Round 2 left 4 critical findings, recorded as unresolved:
1. **Multiple replicas:** the repo deploys 2 replicas (3 in production). Each one loads providers, settings and the default model once at startup and never refreshes them.
2. **Free-text controls:** the schema panel's CSV and raw-JSON fields, plus about a dozen hand-written free-text controls, still have no build item.
3. **Duplicate system picker:** `llm.default_provider` is still a separate text control for the system default.
4. **Unsafe fallback:** the planned last-known-good fallback would let a server start on an older, weaker security config after a deliberate security refusal.

**Separate from this phase:** `IMPECCABLE_BIN` in your own `~/.claude/settings.json` pins the Impeccable engine to 0.1.17, so `impeccable update` won't change what runs. I haven't touched your settings; moving that pin is your call.

The blocking question: do realtime model changes need to be correct across multiple UAR replicas sharing one remote database this phase, or only within a single process?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T19:43:16.298647Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
