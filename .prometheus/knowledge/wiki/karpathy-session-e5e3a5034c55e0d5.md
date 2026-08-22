---
type: SessionRecord
id: karpathy-session-e5e3a5034c55e0d5
title: Karpathy session e5e3a5034c55
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-12T09:47:54.130730+00:00
created_at: 2026-08-12T09:47:54.130730+00:00
updated_at: 2026-08-12T09:47:54.130730+00:00
revision: 0
---

## Delta

GAP-02 and GAP-03 are specified and both validate strictly. GAP-05 is held.

## What I wrote

| Change | Capability | Validates |
|---|---|---|
| [gap-02-jwks-token-verifier](openspec/changes/gap-02-jwks-token-verifier/proposal.md) | `jwt-hardening` (ADDED) | ✅ strict |
| [gap-03-a2a-tenant-partitioning](openspec/changes/gap-03-a2a-tenant-partitioning/proposal.md) | `tenant-isolation` (new) | ✅ strict |

## Two things I got wrong and corrected mid-stage

**I declared GAP-02 as "Modified Capabilities."** It validated anyway — then I checked `openspec/specs/` and found `jwt-hardening` **doesn't exist there**. It's declared only by the unarchived `harden-jwt-defaults`. So `ADDED` was right and my proposal header was wrong. Validation passing did not mean I'd declared it correctly.

**"The requirements are disjoint" was too clean.** Names are disjoint, but `harden-jwt-defaults`'s *"full claim validation"* and my *"signature validity alone does not establish token validity"* cover the same behaviour. They agree, so nothing contradicts — but two requirements over one behaviour with no ordering is exactly what makes an executor guess. I added an explicit precedence rule: **mine governs the JWKS lane, theirs governs the shared-secret lane and config surface.**

## What the specs encode

- **Order is enforced in the tasks, not just described.** GAP-03 task 0.2 requires confirming GAP-02 complete before starting, because a tenant read from an unverified token is worse than no tenant field.
- **The vocabulary collision is written into GAP-03's proposal.** `lib.rs:120` calls per-user credentials "multi-tenant." Your ruling that `tenant_id` ≠ `sub` is recorded there, along with the recommendation to rename `multi-tenant-isolation` → `user-data-isolation` — flagged as *not sanctioned in this change*, with a stop condition if Codex reaches for it.
- **Both changes carry negative controls.** A fail-closed assertion nobody has seen fail proves nothing — that's the discipline that made last phase's L4 test credible.
- **GAP-03 converts the C-21 exclusion rather than leaving it.** [capability_cases.rs:873](tests/integration/live/capability_cases.rs:873) was written to fail when the gap closes; task 3.1 makes that conversion in-scope.

Still to do before handoff: `EXECUTION-CONTRACT.md` (order, precedence, verbatim verification command, stop conditions), then the plan stage. The verification target exists — I checked rather than quoting a command from memory.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-12T09:46:58.452470Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
