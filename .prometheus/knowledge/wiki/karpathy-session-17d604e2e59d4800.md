---
type: SessionRecord
id: karpathy-session-17d604e2e59d4800
title: Karpathy session 17d604e2e59d
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-22T10:59:07.730564+00:00
created_at: 2026-11-22T10:59:07.730564+00:00
updated_at: 2026-11-22T10:59:07.730564+00:00
revision: 0
---

## Delta

All three round 3 fixes are written and now with three independent reviewers: one adversarial, one design, one accessibility. Nothing has been run or committed.

- **Chat test identity:** the runtime writer traced the setting through the config loader. By reading, the test setup meets every condition for the local owner, and it found no S1 step that would break or pass for the wrong reason.
- **Burst 100, two additions:** the native install's default config still shipped 20, and the settings page still said the default was 20. I changed both as part of your "burst 100". Existing installs keep whatever value is already in their config.

It also found problems outside round 3, which I logged and didn't fix:
- **The Burst Size and Req/s settings don't affect the limiter.** It reads only the startup config, so changing them in the UI does nothing to throttling. That belongs with the truthful-settings work (S2).
- **Stale docs:** one embedded example config still ships burst 10, two skill reference docs say 20, and the example config states the wrong default for requests per second.

When the reviews return I'll fix what they find and commit locally. The long test run still needs your answer: **should I build one combined rc.6 candidate here, taking #539 back from Codex ("Yes, combine them"), or finish S1 first ("S1 first")?**

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-22T10:46:12.693991Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
