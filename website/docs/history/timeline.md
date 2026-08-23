---
sidebar_position: 3
title: Architecture Timeline
description: Follow the dated changes that produced UAR's current architecture.
source_records:
  - docs/adr/index.md
  - .prometheus/decisions.md
  - .kbd-orchestrator/phases/uar-1-0-readiness/reflection.md
current_authority: /docs/history/timeline
---

# Architecture timeline

## 2026-07 — establish the runtime contracts

- **July 11:** React 19 became the canonical first-party frontend. Earlier
  HTMX/Web Component plans remained as dated research rather than product truth.
- **July 13–14:** ADRs established configuration secrecy, central errors, SDK
  licensing, supply-chain intent, RAG citations, A2UI ownership, documentation,
  and the ADR process itself.
- **July 22–25:** embedded operation gained transport-free AG-UI, host-owned run
  policy, effective model backfill, conversation policy, and host-supplied
  history boundaries.

## 2026-08 — reconcile intent with delivered behavior

- **August 7:** ADR-0017 replaced the AGPL/commercial split with MIT for the
  complete code repository. The commercial boundary moved from copyleft to
  services; the older rationale remains visible.
- **August 7–8:** the frontend migration established one design authority,
  retained a scoped Base UI divergence, and enforced the Flat 2.0 direction
  while disclosing remaining legacy debt. The old purple guidance ceased to be
  current authority.
- **August 9:** routine verification moved entirely local. GitHub Actions became
  deployment execution and deployment validation only.
- **August 14:** the JWT choice moved from the interrupted AWS-LC spike to
  RustCrypto, with UAR owning first provider installation because upstream could
  not identify an already-installed provider safely.
- **August 15:** skill enablement became durable and most-specific-wins, while
  API-created skill provenance was separated from configuration reconciliation.
- **August 21–22:** shutdown deadlines and the dated Rust image toolchain were
  corrected after operational evidence contradicted prior assumptions.
- **August 22:** duration-only synthetic soak evidence was rejected for inference
  claims. UAR 1.0 closed on five bounded real-model functional paths instead of
  presenting the cancelled release tail as passed.
- **August 23:** the documentation program replaced competing/placeholder Pages
  output with one source-classified branded portal and explicit history.

## Reading the sequence

Dates establish order, not quality. Some later decisions reverse earlier ones;
others narrow what an earlier decision was allowed to prove. For **current
authority**, follow the links in the current product and architecture guides.
The timeline itself is documentation-source evidence only and carries no
runtime, release, deployment, or cross-profile verdict.
