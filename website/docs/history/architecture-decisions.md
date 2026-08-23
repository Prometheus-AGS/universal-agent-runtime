---
sidebar_position: 2
title: Architecture Decisions
description: Index retained UAR ADRs and understand what their status proves.
source_records:
  - docs/adr/index.md
  - docs/adr/0017-relicense-runtime-to-mit.md
  - docs/adr/ADR-007-react-first-frontend.md
current_authority: /docs/history/architecture-decisions
---

# Architecture decisions

## Accepted does not mean delivered

An accepted ADR records an architectural choice. Delivery still depends on the
current source, specification, and profile-specific evidence. ADR status does not
certify deployment, release, runtime behavior, or transfer across profiles.

## Retained ADR index

| Record | Decision | Date | Status |
|---|---|---:|---|
| ADR-0001 | Record architecture decisions | 2026-07-14 | Accepted |
| ADR-0002 | Dual-license runtime under AGPL and SDKs under MIT | 2026-07-13 | Superseded by ADR-0017 |
| ADR-0003 | Set a 60% coverage baseline | 2026-07-13 | Accepted |
| ADR-0004 | Introduce a central UAR error enum | 2026-07-13 | Accepted |
| ADR-0005 | Use config-rs, schemars, and secrecy | 2026-07-13 | Accepted |
| ADR-0006 | Adopt the supply-chain target | 2026-07-13 | Accepted; target is not certification |
| ADR-0007 | Release SDKs as 1.0 under MIT | 2026-07-13 | Accepted; registry availability is separate |
| ADR-0008 | Emit RAG citation streams | 2026-07-13 | Accepted |
| ADR-0009 | Vendor the A2UI web core | 2026-07-13 | Accepted |
| ADR-0010 | Build the UAR A2UI renderer on the web core | 2026-07-13 | Accepted |
| ADR-0011 | Host a unified Docusaurus portal | 2026-07-14 | Accepted; publication evidence is separate |
| ADR-0012 | Keep the AG-UI adapter transport-free | 2026-07-22 | Accepted |
| ADR-0013 | Resolve embedded run policy and expose host administration | 2026-07-24 | Accepted |
| ADR-0014 | Backfill the effective-policy model | 2026-07-24 | Accepted |
| ADR-0015 | Add embedded conversation-policy administration | 2026-07-24 | Accepted |
| ADR-0016 | Seed embedded sessions from host history | 2026-07-25 | Accepted |
| ADR-0017 | Relicense the whole repository to MIT | 2026-08-07 | Accepted; supersedes ADR-0002 |
| ADR-007 | Make React 19 the canonical first-party frontend | 2026-07-11 | Accepted; supersedes HTMX-first product guidance |

The [repository ADR index](https://github.com/Prometheus-AGS/universal-agent-runtime/tree/main/docs/adr)
contains the full rationale and consequences. The non-padded `ADR-007` name is
retained because renaming history would break existing references; its identity
is distinct from `ADR-0007`.

## Cross-cutting decisions outside ADRs

Some decisions were recorded during execution because they resolved an observed
blocker: RustCrypto ownership for JWT, local-only routine verification, genuine
model inference as the inference evidence boundary, and source-first Pages
publication. They are indexed in the [corrections ledger](./corrections) and
mapped to retained evidence by the architecture-history manifest.

## Current authority

Use [Architecture](/docs/architecture/intro) for current runtime structure,
[Security](/docs/security/authentication) for authentication boundaries,
[Profiles](/docs/architecture/profiles) for transfer limits, and
[Deployment](/docs/deployment) for operational behavior. History never widens
those claims.
