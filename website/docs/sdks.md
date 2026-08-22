---
sidebar_position: 5
title: SDKs
---

# SDKs

The repository includes client SDK source packages for Rust, Python, and
TypeScript. All three are versioned at 1.0.0 under the MIT license and cover the
runtime's HTTP and streaming interfaces. Registry publication is release-ordered;
the commands below apply after the corresponding package is published.

| Language | Package | Registry command after publication |
|---|---|---|
| Rust | `universal-agent-runtime-sdk` | `cargo add universal-agent-runtime-sdk@1` |
| Python | `universal-agent-runtime-sdk` | `pip install universal-agent-runtime-sdk` |
| TypeScript | `@prometheus-ags/universal-agent-runtime-sdk` | `npm install @prometheus-ags/universal-agent-runtime-sdk` |

Choose the guide for your language:

- [Rust SDK](./sdk-rust/intro)
- [Python SDK](./sdk-python/intro)
- [TypeScript SDK](./sdk-typescript/intro)

The Rust SDK's optional embedded mode depends on the runtime crate. Its
publication therefore waits for the runtime and the runtime's internal crate
prerequisites. No SDK page is evidence that a registry package is live today.
