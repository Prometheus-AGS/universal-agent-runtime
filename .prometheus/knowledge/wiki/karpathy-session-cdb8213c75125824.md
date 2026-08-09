---
type: SessionRecord
id: karpathy-session-cdb8213c75125824
title: Karpathy session cdb8213c7512
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-08T13:48:52.635468+00:00
created_at: 2026-08-08T13:48:52.635468+00:00
updated_at: 2026-08-08T13:48:52.635468+00:00
revision: 0
---

## Delta

Both real reviewers ran. The result changed the plan more than round 1 did.

## First: the gateway

**Working the whole time.** Process started Aug 6 15:30 — before this session. Config mtime Aug 5. `~/.env` from Nov 2022. I only ever read from it.

The key was in `~/.prometheus/kbd/secrets.env`, and the proxy config **says so in a comment I read past** while quoting its other comments about auth. With it: HTTP 200, five models, `MiniMax-M3` and `k3` both present.

I also nearly reported two fabricated defects — a "TOML syntax error" that was my own `sed` doubling `=`, and an "unset env var" that `ps eww` showed was set. Caught both before telling you. The later 400s were `temperature: 0.2`; **k3 accepts only `1`**.

## The verdicts

**k3 judge: INSUFFICIENT.**

> *"The instrument is sound; the framing the question asks me to endorse is not."*

Stack the limits — no L4 (structural), no semantics, stub LLM, no tenant isolation, one profile — and what's left is *"on server-full, against a fake LLM, in a throwaway database, endpoints return correctly-shaped responses."* That's a smoke matrix. Its decisive point: *"stakeholders read headlines and footnotes get stripped in the first forward."*

**Renamed:** `Capability smoke results — server-full profile, stub provider, no persistence, no semantics`. No aggregate percentage, no runtime verdict.

## MiniMax-M3 killed my Q2 fix

I'd proposed deriving the UI denominator from `SPECIFICATION.md` instead of the frontend. The critic:

> *"Same author. Same brain. Same judgment surface. The original flaw was epistemic ('who derives the denominator?'), not procedural ('which document do they read from?')."*

Sharper than it first reads: **I wrote that spec, in this session.** Deriving a denominator from my own document to measure a frontend whose capability list I also derived isn't independence. Q2 now reports the defect scan only; any coverage denominator **must be authored by someone other than me**.

## And my L3 labels were inflated

> *"The test is asking 'did my code parse my own canned output?' That is the definition of L2."*

Its C-03 example: routing broken, fixture hardcodes the right model, shape assertion passes. I'd conceded this for C-14 only — it applies to **C-03, C-04, C-05, C-08, C-09**. Stub-backed results are now labelled **L2-wired**. C-12/C-13 report *"L4 unverifiable — does not establish persistence."*

## Why round 2 mattered

Round 1 (same-model) found real flaws — the 8-path OpenAPI literal, the 200+index.html false pass, the profile bug. Round 2 found that **my fix wasn't a fix**, that **stub L3 is L2**, and that **the name was the core problem**. Round 1 couldn't see those because it shares my priors.

That's the empirical case for model diversity, and it's why I shouldn't have treated the same-model round as sufficient.

Q1 can proceed under the corrected labels. Q2 needs an external denominator author — your call.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-08T13:43:43.663197Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- TEST_EXECUTION_REPORT.md
