# Design

## Context

UAR currently advertises protocol capabilities but does not identify the logical
runtime instance that answered. The C03 deployment binding already records a
`runtimeInstanceId`, required capabilities, workspace scope, and opaque model and
storage references, but it does not compare those requirements with the live
process. Run creation likewise cannot distinguish a new placement from a
reattachment or a migration request.

The trusted host owns service inventory, credentials, process supervision, and
selection. UAR owns only the description and enforcement of its own configured
identity. This change therefore adds no cross-instance registry, discovery
daemon, fallback launcher, or second supervisor.

## Goals and non-goals

Goals:

- expose one stable configured identity aligned with the existing A2A identity;
- describe profile, ownership, locality, endpoint roles, capabilities, and
  opaque references without exposing credentials;
- refuse incompatible deployment bindings and run placement before execution;
- retain the effective service binding with the run for inspection and resume;
- distinguish new placement and reattachment while refusing migration.

Non-goals:

- selecting among instances, supervising processes, or storing host inventory;
- moving a live run between instances;
- transferring or resolving credential values through UAR discovery;
- changing the owner of conversation history or service lifecycle.

## Decisions

### One identity authority

`service_instance.instance_id` is the canonical configured identity. The
existing `a2a.instance_id` is an alias: either may supply the value, but startup
configuration is invalid when both are configured differently. The same
effective identity is used by A2A contracts, capability discovery, collaboration
binding validation, and run admission.

### Additive discovery contract

`GET /api/uar/capabilities` retains its existing response and adds:

- `instance: { id, profile, workspace_location }`;
- `endpoints: { runtime, administration, models, console }`;
- lifecycle ownership and opaque reference metadata;
- placement support and structured compatibility information.

Endpoint values are credential-free absolute URLs. References are opaque host
store identifiers; UAR neither resolves nor returns credential material.

### Explicit admission and binding

Clients may add a service placement expectation to new-run and resume requests.
UAR compares the expected identity, profile, locality, required capabilities,
endpoint roles, and optional binding identity with the live descriptor. A new
run accepts only `new`; a resume accepts only `reattach`; `migrate` is explicitly
unsupported. Omission preserves legacy clients.

The resulting effective service binding is captured in run context and returned
by run inspection. Reattachment must match the source run's captured instance
and optional binding identity. UAR never starts local fallback work after a
placement refusal.

### C03 binding integration

Deployment-binding preflight evaluates `runtimeInstanceId` and
`requiredCapabilities` against the same live descriptor. Required mismatches
produce required-unsupported diagnostics and make activation inadmissible.
Effective receipts capture the service binding and are revalidated immediately
before execution and tool claims.

## Risks and trade-offs

A legacy deployment that does not configure either instance identity can keep
using ordinary requests, but cannot satisfy an identity-constrained C04 request
or activate a C03 deployment binding. This preserves compatibility without
pretending an ephemeral process has a stable identity.

Endpoint equality is strict when a client supplies expected endpoints. Hosts
behind a proxy must configure advertised endpoint roles rather than relying on a
listener address that clients cannot reach.

## Integration boundary

The Boss and BossFang store inventory, protected credential references, desired
placement, and managed/external lifecycle state. They first inspect the live UAR
descriptor, then submit the same expectation with a run. UAR verifies that the
selected process still matches at admission time. This is a two-party contract,
not shared mutable inventory.

