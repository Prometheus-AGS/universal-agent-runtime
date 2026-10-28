---
type: SessionRecord
id: karpathy-session-2fa0222e7618edb4
title: Karpathy session 2fa0222e7618
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-27T23:55:23.014808+00:00
created_at: 2026-10-27T23:55:23.014808+00:00
updated_at: 2026-10-27T23:55:23.014808+00:00
revision: 0
---

## Delta

Your five decisions (D-1 to D-5) are recorded in `plan.md`. A fresh critic re-reviewed revision 2 and found it isn't ready to build: 3 critical defects and 9 warnings. I've stopped before implementation, as you asked.

**Critical**
1. **Agents can't send custom components at all.** The `a2ui_render` tool's schema allows only the 9 built-in component names, and that schema is compiled once when the tool is registered. My plan updated only the tool's description and catalog list. Every agent call using a template or host component would be rejected before it runs, so I21 and I22 could never pass.
2. **The entity components don't fit the props format I defined.** EntityCard has nested objects such as `accessibility` and `actions[].action`, and several other entity components nest actions inside list items. Seeding all 7 would either fail or have to bypass the check, contradicting my own spec.
3. **The action echo has nowhere to land.** The state snapshot the route starts from has no `actions` field, so the echo of a user's action would fail to apply.

**Warnings, in short:**
- Continuation runs couldn't update a surface from an earlier run. That would also break plain built-in surfaces, which work today.
- My template-expansion IDs would break a surface's required `root` component.
- A refused action after the run is reserved would lock the whole thread.
- Several files have no owner, and two citations point to the wrong file.
- Saved presentations would start accepting host components.
- Four gaps in the security model: an agent with the terminal tool could call the template endpoints when authority is switched off, templates could shadow host component names, template names leak into tool-call events, and action props are left unspecified.
- Startup seeding would never apply an updated host schema.
- Many spec scenarios have no test task.
- Several line numbers in the dedicated-route file are wrong.

**Fixes the critic confirmed from revision 1:** no internal run IDs on the route, the two added message entry points, separate record types for the new components, catalog-scoped names, the reusable admin-key check, and consistency with the chat "invalid surface" rule. It also confirmed that reserving the run first does guarantee a single winner. Exactly-once actions, the per-surface catalog tracking and file ownership are only partly fixed.

**The critic's sharpest point:** with verification held until the end, the first critical defect would only have surfaced at the final gate, about 6 agent-days in.

The full findings, with file and line evidence, are appended to the [plan](.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/agui-a2ui-projection-and-custom-components/plan.md), and the session log is updated. Nothing is committed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-27T23:54:53.239582Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
