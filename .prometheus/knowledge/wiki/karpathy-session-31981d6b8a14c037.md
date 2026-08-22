---
type: SessionRecord
id: karpathy-session-31981d6b8a14c037
title: Karpathy session 31981d6b8a14
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-12T10:09:44.553146+00:00
created_at: 2026-08-12T10:09:44.553146+00:00
updated_at: 2026-08-12T10:09:44.553146+00:00
revision: 0
---

## Delta

Recommendation. What you described is already ~80% specified — but in three separate changes, none of which is aware of the others, and one that contradicts your first requirement.

## What already exists

| Your requirement | Where it lives | State |
|---|---|---|
| Built-ins non-deletable | `add-skill-kind-and-origin` — `origin=Builtin` → 409 | **8/11 done** |
| Enable/disable at global, agent, conversation | `fix-skills-scope-semantics` → capability `skill-governance` | 0/5, **spec'd** |
| Config survives restart | Same, task 1.3 | 0/5 |
| Config changes take effect live | `skill-hot-reload` — already an archived capability | Exists in `openspec/specs/` |

`fix-skills-scope-semantics`'s delta reads almost verbatim as your second and third bullets: *"non-deletable and disableable at global, per-agent and per-conversation scope; disable state SHALL survive restart."*

## The one real conflict

Your first bullet — **built-ins live in permanent storage** — contradicts `add-skill-system-submodule`'s in-memory default. Everything else is consistent with the existing specs; this is the single decision that needs to change, and it's the root cause of GAP-05.

Two consequences worth stating:

- `fix-skills-scope-semantics` task 1.3 says *"merge persisted enabled-state over builtin re-registration at startup."* That's a **workaround for in-memory builtins** — persisting the config separately from the skill. If builtins are rows, that task mostly dissolves; you'd just not overwrite the row.
- **GAP-05 dissolves too.** If builtins are in the database, the embedded path gets them from `SkillService::initialize` via `DatabaseStorageProvider` — the same path that already loads persisted skills. No SDK-side registration call needed.

## What's genuinely new in your description

Your fourth bullet: **config file changes merge into the DB on restart, including deleting non-builtin skills removed from config.** I found no spec for that. It's a reconciliation pass — config is authoritative for provisioned skills, DB is authoritative for user-created ones — and it needs care, because "delete rows whose file disappeared" can destroy user data if the file-vs-user distinction is wrong.

## My recommendation

**Don't specify GAP-05.** Specify the model you just described, as one change that supersedes it:

> **`skill-persistence-and-governance`** — builtins are durable rows with `origin=Builtin`, non-deletable; scoped enable/disable at global/agent/conversation, durable and live; startup reconciliation merges config into DB, deleting only non-builtin skills absent from config.

Then GAP-05 becomes a *verified consequence*, not a task: a test asserting the embedded path sees builtins.

**Sequencing** — and this is the part I'd push back on doing all at once:

1. Land `add-skill-kind-and-origin` (8/11, three tasks left). It defines `SkillOrigin`, which everything else keys off.
2. Then this change, absorbing `fix-skills-scope-semantics` — I'd **supersede** it rather than run both. Two changes writing skill persistence with no precedence is R-3 again.
3. GAP-05's spec amendment lands with it.

**Phase placement:** this is bigger than `uar-1-0-readiness`'s remaining scope and touches a different subsystem than GAP-02/03. I'd make it its own phase rather than widen this one — GAP-02/03 are ready to hand to Codex now, and holding them behind a skill-storage redesign delays working code for no benefit.

**The uncomfortable part:** superseding `fix-skills-scope-semantics` means declaring a 0/5 change obsolete that someone wrote deliberately, and its `skill-governance` capability would need to be either reused or retired. Reusing it is cleaner — same name, extended requirements. But that's a call about someone else's work, and I'd want it explicit rather than silently absorbed.

Nothing written. Want me to draft that change, and if so — new phase or widen this one?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-12T10:09:27.584476Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
