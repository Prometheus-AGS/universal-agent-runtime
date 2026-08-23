---
sidebar_position: 99
title: About UAR
description: Project identity, version, licensing, profiles, and authoritative project links.
source_records:
  - Cargo.toml
  - LICENSE
  - LICENSE-CC-BY-4.0.md
  - docs/product-support-matrix.json
  - frontend/src/pages/about-page.tsx
current_authority: /docs/about
---

# About Universal Agent Runtime

Universal Agent Runtime is a Rust/Axum boundary for governed agent execution,
provider routing, tools, retrieval, memory, typed events, and declarative agent
UI. The first-party operator application uses React 19 and TypeScript.

## Current repository identity

| Field | Repository value |
|---|---|
| Runtime version | `1.0.0` |
| Runtime and SDK code license | MIT |
| UAR-authored documentation license | CC-BY-4.0 |
| Source repository | [Prometheus-AGS/universal-agent-runtime](https://github.com/Prometheus-AGS/universal-agent-runtime) |
| Support authority | [Product support matrix](/docs/architecture/profiles) |
| Security reporting | [Security guide](/docs/security) |

The version is source metadata, not proof that a registry artifact, container,
desktop package, mobile build, or deployed documentation route is available.
Those claims require their own release or deployment evidence.

## Profiles

- `server-full` is the packaged server/sidecar composition with the React
  operator application and full named server capabilities.
- `minimal` is a dependency-light headless server profile. A `server-full`
  result does not transfer to it.
- `embedded-mobile` is a transport-free host-integration profile. The host owns
  presentation, lifecycle, connectivity, persistence injection, and platform
  certification.

The React application's `/about` route displays client-authored capability text
and, when reachable, `/healthz` status/version data. That screen is an operator
summary. Health does not certify inference, tools, persistence, deployment, or
another profile.

Start with [Why UAR exists](/docs/intro), [Architecture](/docs/architecture/intro),
[Installation](/docs/installation), and [Contributing](/docs/contributing/intro).

