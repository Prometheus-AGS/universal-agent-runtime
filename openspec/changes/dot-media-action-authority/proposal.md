## Why

Issue #386 requires UAR to govern media effects initiated by the Dot coordinator and its workers. A disconnected call can have taken effect even when UAR receives no result.

## What Changes

- Add explicit scoped media confirmation, expiring grants, narrowed worker delegation, and durable media effect receipts.
- Add trusted-host media operations and a deterministic mock peer. Preserve the unknown outcome after a lost result and prevent automatic replay.
- Record implementation and local verification in the issue change record and Karpathy session log.

## Capabilities

### New Capabilities

- `dot-media-action-authority`: host-mediated media approval, dispatch, and reconciliation.

## Impact

Adds collaboration API resources and backward-readable catalog fields. Host adapters own confirmation UX, worker identity, transport and processor credentials. Initial delivery simulates outbound calls and human transfer through the mock peer.
