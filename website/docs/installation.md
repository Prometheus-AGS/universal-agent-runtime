---
sidebar_position: 11
title: Installation
description: Install UAR from a verified release artifact or a complete source checkout.
source_records:
  - docs/DEPLOYMENT.md
current_authority: /docs/installation
---

# Installation

## Boundary statement

**Installation puts an artifact or source build on a machine; it does not prove
registry publication, provider inference, persistence, or production health.**
Choose the profile and acquisition boundary first, then perform a functional
check through that exact composition.

## Choose a profile

| Profile | Intended boundary | What the host must supply |
|---|---|---|
| `server-full` | complete server release composition with UI, API docs, A2A, Cedar, telemetry, local models, document intelligence, and WASM | configuration, secrets, writable persistence or remote services |
| `minimal` | smaller HTTP/SSE server with embedded SurrealDB | configuration, provider path, and storage location |
| `embedded-mobile` | transport-free Rust library for iOS, Android, and embedding hosts | persistence, inference/provider metadata, lifecycle, transport, and platform packaging |

Custom additive feature sets are valid builds but are not one of these named
profiles unless their exact composition matches.

## Release artifact boundary

The repository defines signed release manifests, checksums, SBOMs, provenance,
archives, and digest-addressed images. Before installing a release, retrieve the
asset from the release page or registry and follow the
[release verification guide](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/release-verification.md).

A `v1.0.0` tag, a Compose image string, or package version metadata is not by
itself publication status. Confirm the asset exists, verify its manifest and
digest, and pin that immutable digest or verified archive.

## Source prerequisites

The checkout pins the Rust channel and targets in `rust-toolchain.toml` and pins
pnpm `11.15.0` in the root package manifest. Use Node.js 20 or newer for the
portal; the release container uses Node.js 24. Docker/BuildKit is required for
the polyglot image path.

The repository also contains Git submodules. Initialize them before Cargo or
frontend work; a missing submodule can surface as an unrelated manifest or
package error.

## Source build

```bash
git clone --recurse-submodules \
  https://github.com/Prometheus-AGS/universal-agent-runtime.git
cd universal-agent-runtime
git submodule update --init --recursive
corepack enable
corepack prepare pnpm@11.15.0 --activate
pnpm install --frozen-lockfile
pnpm -C frontend install --frozen-lockfile
pnpm -C frontend --filter @prometheus-ags/prometheus-entity-management build
pnpm build
cargo build --locked --release --no-default-features --features server-full
```

The release image also carries the documented polyglot skill/component
toolchains. A host-only source build need not install every toolchain unless it
will compile those components locally.

## Local server start

`config.embedded.yaml` is a loopback development preset with embedded
SurrealKV and anonymous access. Start it only on the local machine:

```bash
CONFIG_FILE=config.embedded.yaml \
  cargo run --locked --no-default-features --features server-full \
  --bin universal-agent-runtime
```

Set a real provider/model and credential through the environment before making
inference requests. A successful `/healthz` proves liveness only; `/readyz`
checks configured dependencies. Genuine inference requires a request through
UAR to an actual loaded or remote model and an observed model response.

## SDK source use

The three SDKs have independent locks and guides:

- [Rust SDK](./sdk-rust/intro.md)
- [Python SDK](./sdk-python/intro.md)
- [TypeScript SDK](./sdk-typescript/intro.md)

Use their source-checkout commands when registry availability is not separately
verified.

## Profile limits

The command above builds `server-full` for the current host. It says nothing
about `minimal`, iOS, Android, another CPU architecture, a container, or a
cluster. `embedded-mobile` requires platform-specific host integration and
cannot be installed as a standalone server.

Next: [Deployment](./deployment.md).
