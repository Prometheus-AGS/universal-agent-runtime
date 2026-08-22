---
type: SessionRecord
id: karpathy-session-0f67a328e78b4cb5
title: Karpathy session 0f67a328e78b
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-12T09:41:08.088572+00:00
created_at: 2026-08-12T09:41:08.088572+00:00
updated_at: 2026-08-12T09:41:08.088572+00:00
revision: 0
---

## Delta

The scan paid for itself. It found a direct contradiction with GAP-05 that I would otherwise have shipped to Codex.

## The scan changed GAP-05

`add-skill-system-submodule` is **13/16 done** and already built `builtin_loader.rs` — the thing at `server.rs:511`. Its proposal records a design decision:

> *"registering all builtins via `SkillService::register_builtin(skill)` (new method that bypasses normal storage and marks the record **in-memory only**…). **Default to in-memory to avoid migration churn.**"*

**In-memory-only registration is exactly why the embedded path has no builtins.** GAP-05 is not an oversight to wire up — it's a consequence of a deliberate choice that's already ~80% implemented. A change telling Codex to "call the two existing functions from the SDK path" would contradict a live design decision in a nearly-finished change.

`fix-skills-scope-semantics` (0/5) compounds it: *"builtin global-disable is overwritten on boot"* — the same in-memory re-registration, surfacing as a different bug.

## Also material

**`conformance-close-capability-gaps` is 15/15 DONE** and its C-21 exclusion is the thing GAP-03 closes. That's the self-invalidating exclusion from last phase working as designed — but it means GAP-03's spec must reference it so the exclusion flips rather than lingering.

**`fix-sidecar-loopback-auth` is 5/6** and deliberately relaxes JWT for the loopback sidecar. My OQ-1 fix (honouring `jwt_required` at `middleware.rs:85`) could break it if written naively. They're compatible — the sidecar sets the flag explicitly, the bug is that the flag is *ignored* — but the contract must say so, or Codex hits a passing test that suddenly fails and can't tell whether it's the fix or a regression.

**`postgres-credential-store` is 7/8** and its proposal says *"multi-tenant deployments"* — worth a look for an existing tenant convention before I invent a claim name.

## Revised recommendation

D-2 is settled (`tenant_id` ≠ `sub`), so:

| Gap | Change |
|---|---|
| GAP-02 + OQ-1 | **Extend `harden-jwt-defaults`** (0/3, R-4 already written) |
| GAP-03 | **New change**, new capability `tenant-isolation`, referencing the C-21 exclusion |
| **GAP-05** | **Reopen as a design question, not a wiring task** |

GAP-05 now needs a decision I shouldn't make alone: **do builtins persist, or does the embedded path re-register in-memory at startup?** The submodule change chose in-memory to avoid migration churn; `fix-skills-scope-semantics` says that choice is already causing bugs. Reversing it is a real change with migration cost. Honouring it means GAP-05 is "call `register_builtins` from the SDK constructor" — which is *smaller* than the spec implies but touches `add-skill-system-submodule`'s unfinished 3 tasks.

I'd also suggest **renaming `multi-tenant-isolation` → `user-data-isolation`** when that change is next touched, now that tenant is confirmed distinct. Two capabilities both claiming tenant isolation, one enforcing only user scoping, is the "omission reads as coverage" failure again.

Nothing written to `openspec/` yet. Want me to proceed with GAP-02 and GAP-03 and hold GAP-05 pending that decision?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-12T09:40:53.232507Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
