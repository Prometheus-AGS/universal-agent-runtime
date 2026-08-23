---
sidebar_position: 9
title: SDK Selection
description: Choose a UAR SDK by source, transport, generated-reference, and profile boundary.
source_records:
  - sdks/rust/README.md
  - sdks/python/README.md
  - sdks/typescript/README.md
current_authority: /docs/sdks
---

# SDK selection

## Boundary statement

**An SDK source package is a client implementation, not evidence that its
registry artifact is published or that the target runtime is healthy.** UAR
keeps source availability, generated references, registry publication, and
runtime-profile support as separate facts.

## Choose a source package

| Language | Source package | Primary mode | Generated reference in Pages | Registry publication |
|---|---|---|---|---|
| Rust | `sdks/rust` | typed async HTTP client; optional in-process embedded runtime | rustdoc staged at `/docs/api/rust/` | verify independently before using a registry-only install |
| Python | `sdks/python` | typed async HTTP/SSE client | Sphinx sources exist but are not staged into Pages | verify independently before using a registry-only install |
| TypeScript | `sdks/typescript` | fetch-based JSON/SSE client with Zod response validation | TypeDoc staged at `/docs/api/typescript/` | verify independently before using a registry-only install |

All three package manifests currently carry version `1.0.0`. That value
describes source compatibility; it is not a registry query.

## Shared client surface

The clients cover chat and streaming, tools, structured output, embeddings,
runs and checkpoints, knowledge bases, and ingestion to the extent documented
in each package. Exact method names and response validation differ by language.
Use the generated or source API for the SDK and the
[API reference map](./api/index.md) for the server boundary.

Authentication is bearer-based in all three clients. An SDK option named API
key attaches the supplied value as `Authorization: Bearer ...`; the caller must
provide either a valid UAR JWT or a token accepted by its configured gateway.
The client library does not create a trusted tenant by itself.

## Generated reference

The documentation deployment generates workspace rustdoc and TypeDoc, then
stages only those two trees. Python Sphinx output can be generated locally from
source but is not part of the current Pages artifact.

## Registry publication

Before adopting a registry install command, verify the exact package name,
version, publisher, checksum or integrity data, and release provenance at that
registry. If it is unavailable, use a pinned source checkout and the local
commands in the language guide. Do not infer publication from a Git tag,
manifest, README, or generated docs page.

## Profile limits

Python and TypeScript are network clients for a `minimal` or `server-full`
server. Rust can also link the transport-free runtime through `embedded` and
`embedded-mobile` features, where the host must supply persistence and other
required services. A client request proves only its exercised route against the
named server profile.

Choose [Rust](./sdk-rust/intro.md), [Python](./sdk-python/intro.md), or
[TypeScript](./sdk-typescript/intro.md).
