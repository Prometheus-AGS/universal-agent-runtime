---
sidebar_position: 5
title: Runtime Profiles
description: The concrete capability and evidence boundaries of server-full, minimal, and embedded-mobile.
source_records:
  - openspec/specs/customer-documentation/spec.md
current_authority: /docs/architecture/profiles
---

# Runtime profiles

## Boundary statement

**A profile is a concrete build composition, not a marketing tier.** A feature,
test, or operational result belongs to the profile that contains and exercised
it. No result transfers silently to another profile.

The current Cargo feature graph defines three profiles used throughout this
portal. The default feature is `minimal`.

## Capability matrix

| Boundary | `minimal` | `server-full` | `embedded-mobile` |
|---|---|---|---|
| Composition | `server` + `surreal-backend` | `minimal` plus the full release capability set | `host-persistence` only |
| HTTP and SSE | Included | Included | Not included |
| Built-in SurrealDB | Included | Included through `minimal` | Not selected; host supplies persistence |
| Inference | Configured server provider path | Server providers plus local-model capability | Host-supplied local driver and provider metadata are required |
| Cedar governance claim | Outside this profile's claim | Included | No server-full claim; embedded runtime uses its in-process host composition |
| A2A transport | Not included | Included | Not included |
| Admin UI, telemetry, API docs, WASM runtime | Not included as a profile set | Included | Not included |
| Transport-free library use | No | No | Yes |

## `minimal`

`minimal` is the default UAR feature. Despite its name, it is a server profile:
it enables the Axum server surface and the embedded SurrealDB backend. It is
appropriate when the full release-only capability set is unnecessary.

Do not describe `minimal` as library-only or transport-free. Do not attach the
server-full Cedar, A2A, telemetry, admin UI, local-model, document-intelligence,
or WASM claims to it unless those features are added explicitly and verified as
that new composition.

## `server-full`

`server-full` extends `minimal` with A2A transport, local models, Cedar
governance, response quality, document intelligence, telemetry, generated API
documentation, the admin UI, and the WASM runtime. It is the profile used for
the complete server product claim.

Because it includes `minimal`, it retains HTTP/SSE and SurrealDB. Its broader
feature set also means a check against `minimal` alone is not evidence that
server-full composition, startup, or integrations work.

## `embedded-mobile`

`embedded-mobile` is the transport-free library profile for Android, iOS, and
other embedding hosts. It does not bind a socket, create server application
state, start sidecars, select a built-in database, or silently choose a remote
LLM provider.

The host must supply a local inference driver, matching provider/model metadata,
and an implementation of `PersistenceLayer`. A successful build is therefore a
host-composition boundary: the embedded runtime has real required components,
while optional embedding, MCP, memory, and native-skill services remain explicit
host choices.

## Additive features are custom compositions

Cargo capability features are additive. A consumer can build a composition not
listed in this table. Such a build must be named by its actual enabled features;
it must not borrow a profile's evidence merely because it contains some of the
same modules. `desktop-full`, for example, extends `server-full` with Tauri and
has its own packaging boundary.

## Profile limits

Documentation and verification report these profiles separately:

- `server-full` evidence applies only to the full server composition.
- `minimal` evidence applies only to the default server subset.
- `embedded-mobile` evidence applies only to the transport-free host-injected
  library composition and must identify the target platform where relevant.

Cross-profile statements in this portal describe shared source structure, not a
shared readiness verdict. Read [Protocol boundaries](./protocols) next to see
how the entrances differ.
