---
type: SessionRecord
id: karpathy-session-cb3e9dedaf6e62ea
title: Karpathy session cb3e9dedaf6e
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-14T02:09:48.576058+00:00
created_at: 2026-08-14T02:09:48.576058+00:00
updated_at: 2026-08-14T02:09:48.576058+00:00
revision: 0
---

## Delta

Both done and pushed.

## Binary

**Already current — my earlier claim was wrong.** I said `~/.local/bin/prometheus` predated the label fix; it didn't. The rebuild produced a byte-identical image (`969965d7`), because I'd installed it during the earlier rebuild without registering that.

Proven live rather than by hash: a real phase transition now writes `Project-wide progress: 48 of 79 (implementation, ALL phases)` with the corrected explanation. Tier gate opened deliberately and closed after.

## Postmortem

`.prometheus/postmortems/` was empty — this is its first file, at `0d563f64`.

| Surface | What went there |
|---|---|
| `postmortems/2026-08-13-…` | Full record: mechanism, why every check missed it, root cause, fix, verification |
| `gotchas.md` | Symptom → root cause → why nearly invisible → fix → prevention, pointing to the postmortem |
| `decisions.md` | Decision / Mechanism / Stakes — including the cost: a legitimate foreign directory now fails the install |
| Karpathy record | Via `record-progress.sh`; wrote repo wiki + private superset + `events.jsonl` |
| `wiki/index.md` + `log.md` | Both updated together |
| surreal-memory | `global` (the method rule) + `prometheus-skill-pack` (operational specifics) |

One adjustment: the project-scope memory **failed at 663 tokens against a 512 limit**. I trimmed it to operational essentials rather than dropping it — the postmortem holds the full record either way.

## What the postmortem says that matters most

It records the method failure as the general one, not the installer bug: *the scope of an instrument was repeatedly mistaken for the scope of the problem.* Four times in one session. The rule that would have caught it was already written in this repo's own execution contract, applied to two other guards the same day, and skipped here because a checkmark had already printed.

It also records the **two defects I introduced while fixing this** — the broken generation seal and the false reminder label — rather than quietly repairing them.

## Still open

**The rule change.** You chose "run the gates first," so per §D-6 the text for *completeness claims require a denominator* goes through adversarial review and the sycophancy gate before landing in `AGENT_BASE_RULES.md`. The postmortem's closing line states plainly that until then it is the record, not the rule.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-14T02:09:02.219981Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
