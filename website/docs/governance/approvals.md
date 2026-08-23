---
sidebar_position: 2
title: Resolve Tool Approvals
description: Approve or reject a permitted tool call that requires human review.
source_records:
  - openspec/specs/tool-approval-workflow/spec.md
  - docs/product-surface-inventory.md
current_authority: /docs/governance/approvals
---

# Resolve Tool Approvals

The runtime pauses a tool call only after the effective run policy and Cedar decision allow an approval path.

:::warning Boundary statement
Approval is not an authorization bypass. A denial is terminal, and an approval response cannot override an effective-run or Cedar denial.
:::

## Decision order

1. If the effective run policy is `Deny`, UAR emits `ToolCallDenied` and rejects the call without creating an approval.
2. Otherwise, UAR asks the Cedar engine for a tool decision when one is attached.
3. A Cedar denial emits `ToolCallDenied` and stops.
4. **Allow** executes immediately unless the effective policy is `Ask` or the runtime risk heuristic requires review.
5. **RequireApproval** emits an approval-required event and parks the tool call.

This order prevents an operator click from converting a forbidden action into an allowed one.

## Packaged UI workflow

During a chat, `ToolApprovalDialog` displays the tool name and arguments and offers Approve or Reject. The Runtime Console also has an Approvals view at `/admin/approvals` for pending entries. Both send a decision to `POST /api/uar/runs/{run_id}/approval`.

The response is single-use. After the pending channel is resolved, a second response for the same run finds no pending approval. The browser view is a projection of runtime state, not the authority that executes the tool.

## Wait and cancellation semantics

The runtime waits up to five minutes. Approval continues the tool call. Explicit rejection stops it. A timeout auto-rejects. If the approval channel closes, the call is rejected. Run cancellation removes and closes any pending sender so the parked operation cannot outlive the cancelled run.

Closing a browser is not itself a durable rejection command; it may instead leave the runtime waiting until another surface responds, the channel closes, cancellation arrives, or the timeout expires.

## Observable outcomes

- `ToolCallApprovalRequired` identifies a parked tool call.
- `ToolCallDenied` records an effective-policy or Cedar denial and its reason.
- A delivered approval or rejection is reflected in the runtime entity projection.
- Timeout and channel-close paths reject rather than execute.
- A terminal run outcome remains the authority for whether the run completed, failed, or was cancelled.

These events and logs are not a signed or immutable approval ledger. Retention depends on the active process and telemetry configuration.

## Profile limits

The runtime approval gate applies to `server-full` and relevant `minimal` server execution. The branded dialog and `/admin/approvals` page are `server-full` UI features. In `minimal`, Cedar is absent even though risk or `Ask` can require approval. `embedded-mobile` must supply its own user interaction and cancellation lifecycle if it exposes an approval-capable execution path.

See [governance policy behavior](/docs/governance/overview) and [run inspection](/docs/operations/runs).
