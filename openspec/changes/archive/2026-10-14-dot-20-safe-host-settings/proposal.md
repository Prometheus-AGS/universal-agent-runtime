# Proposal

## Why

Issue #379 requires deny-by-default browser origins, safe shipped host/model/embedder/auth defaults, and one Bearer-admin effective-settings view for Kubo. Source analysis found permissive CORS, implicit remote-capable defaults in some shipped configuration, and per-key settings reads that do not represent the live registry/governance aggregate. Kubo's reader needs effective settings, but its Topology B gate still requires all nine issue items in one published artifact.

## What Changes

Ship an explicit CORS origin allowlist with an empty default, no implicit model selection, local/off memory embedding, loopback native binding and JWT-enabled installable profiles. Shared-host overrides must be deliberate and documented. Add a read-only, secret-free effective-settings aggregate composed from the live provider registry, persisted settings synchronization and governance snapshot. Pin the stable path/schema, revision policy and Bearer-admin claim contract with Kubo before implementing the route; `GET /api/uar/settings/effective` and Kubo's published schema are the recommended starting point.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-service-deployment`: add CORS and shipped network-default behavior without replacing its existing loopback listener contract.
- `provider-model-settings-certification`: add safe model/embedder defaults and an authoritative effective-settings aggregate.
- `jwt-hardening`: add Bearer-admin read authorization and shipped JWT-on posture while preserving verified local development choices.

## Impact

Production ownership is host configuration/shipped profiles, `src/server.rs`, settings API, live provider registry and settings synchronization, with one serial server writer. Runtime UX gains explicit missing-model errors and a truthful effective view; no new UI is required. Provider endpoints and configured defaults remain operator-owned; the new read does not mutate them. Realtime consumers use the aggregate revision, not a second settings authority. KBD workflow state is updated only through the lead's typed import/execution. All product checks wait for the full Dot production boundary.
