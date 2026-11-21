---
type: SessionRecord
id: karpathy-session-b4b94558023d5c92
title: Karpathy session b4b94558023d
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T18:26:31.467975+00:00
created_at: 2026-11-19T18:26:31.467975+00:00
updated_at: 2026-11-19T18:26:31.467975+00:00
revision: 0
---

## Delta

I've recorded that decision in `.prometheus/decisions.md` and marked question 6 resolved in the assessment. Six of the eight questions are now answered.

**What it covers:** settings that are saved but that no code reads come out of the operator console. That list includes `server.*`, `security.*`, `persistence.*`, `llm.model`, sandbox, vision, sycophancy, OCR and the other namespaces in it.
- **What stays:** the YAML keys and the code that reads YAML at startup. A value honored from YAML is still configurable there.
- **One safeguard:** I built the list by reading the source during the assessment, and I haven't proven it key by key. Before a control is removed, Analyze confirms that no code reads it at runtime. If something does read a key, that key moves into the work to wire it up or give it a picker instead.
- **What it costs:** operators lose the UI path for these keys. Any rows already saved for them become dead data, which planning has to either leave alone or clean up deliberately.

Nothing is committed.

Next are the open OpenSpec changes that overlap this phase, listed in §0.5 of the assessment. Should each one be archived, superseded or continued as part of planning, or should they all be archived now?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T18:25:34.726489Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
