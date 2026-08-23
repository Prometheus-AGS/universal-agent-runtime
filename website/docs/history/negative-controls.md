---
sidebar_position: 8
title: Negative Controls
description: Prove that a fail-closed assertion can detect the failure it names.
source_records:
  - .kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md
  - openspec/changes/archive/2026-08-15-gap-02-jwks-token-verifier/evidence/fail-closed-negative-controls.md
  - openspec/changes/archive/2026-08-15-gap-03-a2a-tenant-partitioning/evidence/fail-closed-negative-controls.md
current_authority: /docs/history/negative-controls
---

# Negative controls

A fail-closed assertion observed only passing can be **vacuous**: the test may
always pass, never execute the guard, or inspect the wrong boundary. UAR pairs a
fail-closed result with a deliberate control that is **observed to fail**.

## The five-part record

1. Name the failure the requirement must reject.
2. Make one bounded inversion that should expose that failure.
3. Run the same assertion and retain its non-zero result and relevant output.
4. Perform **exact restoration** and prove the source/fixture returned to its
   original state.
5. Rerun the affected positive assertion and retain the passing result.

Examples include disabling registered-claim validation, collapsing tenant keys,
removing a provider feature, allowing a stale lockfile, or weakening a
documentation privacy rule. The mutation must target the actual boundary. A
different test that happens to fail is not the negative control.

## When it is required

Use a paired control for a fail-closed security rule, denial boundary, guard,
validator, reconciliation safety rule, or claim whose credibility depends on
detecting absence. Do not manufacture a negative control for every ordinary
positive test; the control should name a plausible defect, not add ceremony.

## Safety

Prefer isolated fixtures or a scratch worktree. If source inversion is necessary,
capture the pre-inversion diff, modify the minimum span, run only the affected
assertion, restore exactly, compare the diff, and rerun the positive path. Never
leave the inverted guard in a commit.

## Reporting limit

The control proves sensitivity to the named mutation. It does not prove the
absence of every bypass, another profile, or production deployment. Retain that
limit in the verification row.
