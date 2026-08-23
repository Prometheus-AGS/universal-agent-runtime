---
sidebar_position: 1
title: Architecture History
description: Read UAR's decisions and reversals without confusing history with current product truth.
source_records:
  - docs/adr/index.md
  - .prometheus/decisions.md
  - .kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/reflection.md
current_authority: /docs/history/overview
---

# Architecture history

UAR keeps the records that explain how its current boundaries were reached. The
useful history includes accepted decisions, abandoned plans, implementation
corrections, failed evidence, and changes made after the product contradicted its
own documentation.

## History is evidence, not authority

For present behavior, start with the [runtime architecture](/docs/architecture/intro),
the relevant product guide, current source, and canonical OpenSpec requirements.
History explains why those authorities look the way they do. An accepted ADR or
completed plan is not proof that every described behavior remains shipped.

The public record has five layers:

1. [Architecture decisions](./architecture-decisions) indexes the retained ADRs
   and selected cross-cutting decisions.
2. [Timeline](./timeline) places changes in order.
3. [Corrections](./corrections) preserves positions that were explicitly reversed.
4. [Process provenance](./process-provenance) explains ADR, OpenSpec, KBD, and
   Prometheus history roles.
5. [Testing methodology](./testing-methodology) records how evidence standards
   changed; it is delivered by the next documentation change.

## Selection and limits

The inspected snapshot contained 18 retained ADRs, 546 Prometheus-history files,
51 top-level KBD phase directories, 48 KBD reflections, and 184 OpenSpec change
directories. The public narrative selects decisions that changed current
architecture, public obligations, or evidence policy.

That selection creates **selection bias**. It is a navigable history, not a full
transcript and not a claim that unselected work was unimportant. Raw wiki pages,
event records, conversations, machine-local context, and credentials remain
private synthesis inputs. The uncomfortable consequence is that a concise public
story cannot reproduce every disagreement; traceability depends on maintainers
preserving the version-controlled source corpus.

## Status vocabulary

- **Current** — the selected decision still matches its named present authority.
- **Superseded** — a later decision explicitly replaced it.
- **Historical** — useful context that is neither current product authority nor a
  promise that the proposal shipped.

These labels apply to the record, not to every runtime profile. Product claims
remain bounded separately to `server-full`, `minimal`, or `embedded-mobile`.
