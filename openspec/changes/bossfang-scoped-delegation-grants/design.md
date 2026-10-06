# Design

## Context

See proposal.md for the C14.4 motivation. Existing `SidecarGuard` authenticates
the per-launch stdin credential, checks loopback authority/no Origin, and strips
Authorization before inner layers. Authentication accepts `x-uar-principal`
only when that guard installed `HostAuthenticated`. The full-harness authority
already owns process-local admissions, task identity, subject/tenant/workspace,
digest/revision conflicts, observation, approval forwarding, and cancellation.

## Goals / Non-Goals

The selected principal, explicit workspaces, and selected operations define the
grant. The original host approves issuance; the grantee cannot increase or renew
rights. Existing verified JWT remains the external transport identity. No
credential persists and no new execution loop or durable backend is introduced.

## Decisions

1. `DelegationGrantAuthority` holds redacted 256-bit secrets and captured
   `UserContext` in native memory. It shares the full-harness runtime epoch and
   service instance ID. Monotonic expiry fixes a maximum 900-second lifetime;
   constructing a new runtime authority invalidates every old grant. Persistent
   API keys or forwarding the launch token would widen the approved boundary.
2. Only managed-local server wiring mounts host-authenticated POST
   `/api/uar/delegation-grants` and DELETE `/api/uar/delegation-grants/{id}`.
   The host principal is an authenticated header assertion captured by existing
   middleware; the body accepts only `workspace_ids` and `operations`.
3. The outer guard first checks authority and no Origin, then original host
   credential or scoped grant. A grant installs `DelegationAuthenticated`, never
   `HostAuthenticated`. Authentication rejects a caller principal header and
   inserts the captured grant identity without a JWT/anonymous fallback.
4. Explicit enum operations are `discovery`, `model_read`, `model_completion`,
   and `full_harness_delegation`. Model completion is separately approved and
   permits only POST `/api/chat/completion`. The exact method/path table lives in
   `docs/protocols/scoped-delegation-grants.md`; completion and full-harness
   controls require exactly one matching workspace header. Grants confer no
   `/api/uar/test`, raw run administration, or nested grant issuance authority.
5. Existing full-harness admission and ownership checks receive the captured
   subject and tenant. Cedar and effect authorization continue unchanged;
   paired-host tool-admission requires the original host marker. Unsupported
   steering remains unsupported. Revocation blocks new requests, while work
   cancellation remains an explicit existing run operation.
6. Capability discovery exposes the grant contract only for managed-local host
   and grant requests. Grant principal mode remains `token-subject`, never
   `host-asserted`. OpenAPI adds the DTOs and separate authentication schemes.

## Risks / Trade-offs

- Grant loss or runtime restart requires reissuance through The Boss. This is a
  deliberate private-memory lifecycle, not durable credential recovery.
- Revocation/expiry does not cancel already admitted native work or undo effects.
  The host uses the existing cancellation contract when it intends cancellation.
- A leaked grant carries its selected finite rights until expiry/revocation.
  Boss main and BossFang native memory retain it; renderer/status/log/settings
  surfaces expose no credential.
- Source implementation is not operational evidence. The coordinator runs the
  real sidecar and packaged Boss C14.4 gate after all component code is complete.

## Migration Plan

Ship native grant support together with the Boss/BossFang connection owner.
External JWT clients retain their existing identity and endpoint contracts.
Rollback removes the new connection integration and restarts native UAR so all
previous grants disappear. No persistent migration is needed.
