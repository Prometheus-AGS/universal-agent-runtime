---
sidebar_position: 5
title: Process and Provenance
description: Understand how ADR, OpenSpec, KBD, and retained Prometheus history support public claims.
source_records:
  - docs/adr/0001-record-architecture-decisions.md
  - openspec/changes/archive/2026-08-23-establish-documentation-publication-contract/proposal.md
  - .prometheus/decisions.md
current_authority: /docs/history/process-provenance
---

# Process and provenance

UAR uses four record types because no single artifact answers intent, behavior,
execution order, and learning equally well.

| Record | What it answers | What it does not prove |
|---|---|---|
| ADR | Which significant architecture choice was accepted and why | That every consequence shipped or remains current |
| OpenSpec | Which observable behavior a change must add, modify, or remove | That implementation or verification occurred |
| KBD | Which phase/stage/change was active, ordered, blocked, or completed | That a status projection alone satisfies the underlying requirement |
| Prometheus history | Which decision, defect, constraint, or lesson was retained | That raw working context is safe or suitable for public release |

## KBD and OpenSpec together

OpenSpec owns the behavioral delta. KBD owns the lifecycle around it: Assess,
Plan, Execute, and Reflect. A complete public account can cite both, but neither
replaces current source inspection. KBD projections may also contain machine
context, so the public site uses reviewed synthesis rather than embedding them.

## The private synthesis boundary

The 2026-08-23 snapshot included 546 Prometheus-history files, of which **536**
were generated wiki records. Those files and KBD artifacts are classified
`private-synthesis-only`. A maintainer may use them to locate a decision, then
must verify it against retained decisions, ADRs, OpenSpec, current source, and
the relevant reflection before publishing a summary.

Public output rejects exact raw copies, machine-local paths, credential-shaped
assignments, private-key material, and raw event/session payloads. Wiki records
cannot be named as direct authority because they are unreviewed projections of
other material.

## Traceability model

The architecture-history manifest maps each public decision ID to dated source
records, its disposition, any superseded position, and a current authority. The
site presents the reviewed result. Repository maintainers can audit the mapping
without turning the raw source into web content.

## The uncomfortable limit

Synthesis is interpretation. Counting every source does not remove **selection
bias**, and a source path can remain present after its semantics drift. The
controls prove classification, required corrections, and reference integrity;
they do not prove that the narrative is the only defensible reading. Current
product claims therefore remain subordinate to current source, specification,
and observed behavior.
