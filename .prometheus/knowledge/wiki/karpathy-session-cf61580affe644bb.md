---
type: SessionRecord
id: karpathy-session-cf61580affe644bb
title: Karpathy session cf61580affe6
tags:
- karpathy
- session-learning
sources:
- resource: session:f1444227-3c6c-46fa-b137-2dd9075d415f
generated:
  by: pk/1.11.0
  at: 2026-11-23T10:14:36.930732+00:00
created_at: 2026-11-23T10:14:36.930732+00:00
updated_at: 2026-11-23T10:14:36.930732+00:00
revision: 0
---

## Delta

Yes, mostly. Five of the problems we hit are defects in the Prometheus skill pack or its `prometheus` runtime, not in UAR. Two of them are confirmed from source or output, two are confirmed by what's on disk, and one is a likely cause I haven't proven.

1. **A read-only command can permanently block KBD writes.** Likely cause, not confirmed.
   - The registry entry for this checkout has `readOnly: true`, last updated at `2026-11-23T10:10:15Z`. That's when I ran `prometheus kbd status --json` inside Claude's sandbox.
   - My reading: the runtime probes file locking on every command. Under the sandbox the probe failed with `Operation not permitted`, and the runtime saved "read-only" to the machine-wide registry. After that, every process was refused, including your terminal and an unsandboxed read.
   - If that's right, there are two defects. A read command shouldn't write to the registry, and one failed probe shouldn't stick with no obvious way to re-probe. I only know that `kbd register` might re-probe; I haven't confirmed it.
2. **Adoption lint says "clean" when the contract will fail.** Confirmed in `kbd-contract-lint.mjs`. Without `--status`, `kbd-contract-adopt.sh --lint-only` skips the owner-task check, but real adoption runs it. That gap cost you one wasted signature.
3. **You're asked to sign before the contract is fully validated.** Confirmed from the output. The amendment template prints "Sign with…" without running the full lint first. The script that signed and then adopted was mine, but the pack offers no step that validates before signing.
4. **`kbd-reflect` depends on files the installed pack doesn't ship.** Confirmed. The skill sources `shared/lib/hooks.sh`, `stage-gate.sh` and `bottleneck-guard.sh`, but the installed pack 1.15.0 has none of them under `shared/lib`. So `reflect:after` and `phase:after` can't fire as documented.
5. **Phase progress is tracked in three places that disagree.** Confirmed. For this phase:
   - the runtime says 54 S1 tasks are complete;
   - `progress.json` lists no changes (`changes: []`);
   - the OpenSpec `tasks.md` checkboxes show 0 of 59.

One more is a design gap rather than a bug. A contract-bound phase can only close on PASS evidence for its bound scenarios. When you decide on a different exit, as you did on 2026-11-22, there's no supported way to record it. The only route is to rewrite the contract's scenario set and sign it again, which is what we've been fighting.

These are separate from UAR, so they'd go to `Prometheus-AGS/prometheus-skill-system` as one issue with five parts, or as separate issues. Nothing is filed yet.

The phase close is still blocked on item 1. Your registry needs re-registering from an unsandboxed terminal, and I wasn't allowed to make that change from this session.

Should I draft the skill-pack issue for you to review, and leave the phase close until the runtime is fixed or the registry is re-registered?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f1444227-3c6c-46fa-b137-2dd9075d415f
- Captured: 2026-11-23T09:21:00.907881Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
