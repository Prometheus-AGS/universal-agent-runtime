---
type: SessionRecord
id: karpathy-session-ff96d901578367c1
title: Karpathy session ff96d9015783
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-13T16:47:24.008575+00:00
created_at: 2026-08-13T16:47:24.008575+00:00
updated_at: 2026-08-13T16:47:24.008575+00:00
revision: 0
---

## Delta

Now I have the complete mechanism, and it's a deliberate design choice with an unintended consequence.

## The root cause, in one line

`targetDestination()` at [install-plugin-generation.js:1037](scripts/install-plugin-generation.js:1037):

```js
if (isManagedCopy(primary, target)) return primary;
return path.join(targetRoot, `prometheus-${skill.name}`);   // ← renames instead of failing
```

When something it doesn't own occupies `~/.claude/skills/deep-research`, the installer **silently installs to `prometheus-deep-research` instead** — then verifies *that* path and reports success.

So the install genuinely succeeded, by its own definition. `deep-research` **is** current on disk — under a name nothing looks for. Your April stub sits at the name Codex searches.

**Six skills are affected in `.claude` alone**: `deep-research`, `context-bootstrap`, `entity-skills`, `rust-auditor`, `skill-pack`, `sycophancy-correction`. Seven in `.agents`, five in `.opencode`.

The design intent is defensible — don't clobber files you don't own. The failure is that **avoiding a clobber and reporting success are incompatible**. It should refuse loudly.

## Why my checks couldn't see it

- My 25-skill sample didn't include any of the six.
- My freshness check compares one SHA and greps duplicate names — a renamed install produces neither symptom.
- The installer's own "✅ Verified … installed to all supported user targets" is true of the generation, and says nothing about whether tools can *find* the skills.

## What I'm going to do

1. **Make the rename path fail loudly** — an unowned occupant at a target name is a hard error naming every collision and the exact command to resolve, not a silent rename. If the fallback stays for genuine third-party conflicts, it must exit non-zero and list them.
2. **Add a real completeness gate** to the installer: every skill × every target resolves, by canonical name, to current generation content. Copy targets compared by content hash — that's how `artifact-refiner` drifted in `.codex`/`.minimax` without anyone noticing.
3. **Run it as the last step of every install**, so a broken install can't report success.
4. **Red tests first** — plant each failure mode (unowned dir, stale copy, missing skill, dangling link), prove the gate fails, then fix, then prove it passes.
5. **Resolve the current damage**: move the six stale occupants aside, reinstall, verify all 165 × 14 resolve.

Two things I also owe you: `process` and `react` are **missing from every target** — I need to check whether they're real skills or directory artifacts. And `artifact-refiner` differs between the generation and the copy targets.

Starting with the red tests, so the gate is proven to catch each case before I claim anything.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-13T16:45:10.795941Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
