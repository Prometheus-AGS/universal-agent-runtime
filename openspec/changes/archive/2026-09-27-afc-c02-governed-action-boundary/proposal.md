# afc-c02-governed-action-boundary

## Why

UAR has an exact-invocation admission seam for model tool calls, but authority is still incomplete. The prepared invocation does not bind the canonical payload, concrete resource, grant, lease, and budget revisions; the paired host is not rechecked immediately before UAR claims an effect; direct REST tool execution calls the MCP registry without the seam; and policy loading can replace missing or invalid policy with a permissive engine. Those paths let the same apparent action execute under different authority than the one inspected or approved.

## What changes

- Bind each prepared effect to a stable invocation identity plus canonical resource, payload, policy, grant, lease, and budget digests.
- Extend the existing `the-boss.uar.sidecar/1` admission family with a claim-time revalidation operation. UAR remains the sole owner of claim, dispatch, terminal, interrupted, and uncertain effect lifecycle states; the host supplies admission facts and approval authority.
- Re-evaluate local policy and the host admission immediately before claim after any approval wait.
- Route direct authenticated REST execution through descriptor resolution, schema validation, prepare, admission, claim, dispatch, and finish.
- Make governed server profiles reject missing, unreadable, empty, or invalid Cedar policy rather than synthesizing permit-all authority.
- Retain an explicit, visible local-only bypass only when runtime governance has proved the constrained loopback posture.

## Scope

UAR runtime governance, tool admission, direct tool REST execution, and embedded admission injection. The Boss owns the paired-host endpoint implementation and approval presentation. No protocol family replacement, scheduler transfer, or dependency upgrade is included.

