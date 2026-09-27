## Why

Hosts can inspect UAR's feature list but cannot verify which logical runtime instance answered, which endpoint roles it supports, or whether a deployment binding targets that live instance. Agent Fabric Convergence C04 requires an identity-bearing service contract so hosts refuse a wrong or incompatible runtime before credentials or executable work cross the boundary.

## What Changes

- Extend authenticated UAR capability discovery with a stable configured instance identity, versioned service profile, endpoint-role metadata, workspace locality, and lifecycle ownership posture.
- Add structured compatibility evaluation for expected instance identity, API profile, and required capabilities.
- Enforce the live runtime instance against collaboration deployment bindings and expose the effective service binding in run inspection.
- Keep service inventory, credential references, placement, and managed-process lifecycle in the trusted host; UAR describes and enforces its own instance only.
- Preserve current AG-UI, administration, A2A, and ordinary run clients through additive response fields and optional request constraints.

## Capabilities

### New Capabilities

- `service-instance-placement`: Identity-bearing UAR self-description, compatibility refusal, and effective runtime binding for host-selected placement.

### Modified Capabilities

- `collaboration-package-catalog`: Installed deployment bindings must target the live runtime instance and retain opaque credential and connection references.

## Impact

This affects UAR configuration, `/api/uar/capabilities`, run admission and inspection, collaboration deployment-binding validation, and Rust SDK request/response types. It consumes initiative C04 and the accepted C01-C03/P1 contracts. KBD task C04.1 remains active until the host and BossFang consumers pass the shared acceptance gate.
