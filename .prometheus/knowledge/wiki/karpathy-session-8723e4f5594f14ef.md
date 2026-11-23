---
type: SessionRecord
id: karpathy-session-8723e4f5594f14ef
title: Karpathy session 8723e4f5594f
tags:
- karpathy
- session-learning
sources:
- resource: session:f1444227-3c6c-46fa-b137-2dd9075d415f
generated:
  by: pk/1.11.0
  at: 2026-11-23T12:47:56.499534+00:00
created_at: 2026-11-23T12:47:56.499534+00:00
updated_at: 2026-11-23T12:47:56.499534+00:00
revision: 0
---

## Delta

I wrote the prompt to [handoffs/rc7-codex-start-prompt.md](.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/handoffs/rc7-codex-start-prompt.md). It isn't committed. Paste the file into Codex on the Intel Mac.

The prompt walks Codex through four steps:

1. **Orient.** Read `AGENTS.md`, `versions.toml`, the gotchas file, the rc.6 reflection, the closure packet and #542. Codex treats #542 as a list of proposals, not agreed scope.
2. **Set up the machine.** Clone `main` at `3a574c3e` with the submodules. Install skill pack 1.16.2 or later for Codex and verify it. Register the checkout, and re-probe if it comes up read-only.
3. **Confirm the old phase is closed.** It must show `CLOSED_SUPERSEDED`. If that doesn't show on the new machine, Codex stops and reports rather than recreating anything.
4. **Open rc.7, then assess.**
   - Codex prepares the start-successor manifest and hands you the signing command. You sign it; Codex never does. The proposed phase id is `uar-rc7-2026-11`, which you can change.
   - Codex then assesses #542 against the current code and recommends a single value slice for this phase. That is the main lesson from rc.6.
   - For that slice it lists scenarios that can actually run on the Intel Mac, then stops before any contract is adopted or code is written.

The prompt also carries your standing rules: BDD-only validation, clean builds, sccache, GitHub Actions for deployment only, no test credentials, commit by name, and your approval before any sign, push, tag or publish.

One thing may stop Codex at step 3. The KBD state from this machine may not exist on the Intel Mac. It lives in local app data, not in git. Codex will also stop if the Intel Mac has no allowed-signers entry for your key.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f1444227-3c6c-46fa-b137-2dd9075d415f
- Captured: 2026-11-23T12:47:45.471536Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
