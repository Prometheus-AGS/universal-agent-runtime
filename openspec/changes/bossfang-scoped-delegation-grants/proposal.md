# Proposal

## Why

C14.4 connects BossFang to the selected UAR runtime. Forwarding The Boss's launch
credential would confer the original host's whole authority; the approved
delivery instead requires a bounded native credential for the selected principal
and workspaces.

## What Changes

- Add host-only grant issuance/revocation to managed-local UAR.
- Admit grants only on explicit discovery, model read, model completion, and
  full-harness methods/paths, binding completion and task requests to workspace.
- Capture verified principal at issuance; grant callers cannot assert identities
  or acquire host-only tool-admission, administration, or credential rights.
- Advertise the native credential contract and document its 15-minute expiry,
  revocation, runtime epoch, private-memory boundary, and external JWT separation.

## Capabilities

### New Capabilities

- `scoped-delegation-grants`: Finite process-local credentials for approved UAR
  delegation operations without launch-host authority.

### Modified Capabilities

None. Existing external JWT and full-harness semantics remain authoritative.

## Impact

Affected source: native security guard/middleware, grant API, server wiring,
capability discovery and additive OpenAPI. No dependency changes, new service,
port, durable backend, or native execution loop. Runtime UX in The Boss receives
status/expiry from its main-process connection owner; tokens remain private.
Provider compatibility retains the existing native completion endpoint and
full-harness SSE contracts. Realtime business state stays with existing UAR run
and task authorities. The C14.4 coordinator owns parent KBD bookkeeping; this
component does not hand-edit canonical workflow state.

Approval: the operator's complete C14.4 plan, relayed by the coordinator, includes
these grants and explicitly scoped model completion. This scoped change records
that approved component; it is not a new architecture approval gate.
