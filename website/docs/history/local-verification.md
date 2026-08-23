---
sidebar_position: 9
title: Local Verification and Timing
description: Run UAR checks at the boundary where their evidence becomes useful.
source_records:
  - AGENTS.md
  - .claude/rules/rust.md
  - .claude/rules/typescript.md
  - .prometheus/decisions.md
current_authority: /docs/history/local-verification
---

# Local verification and timing

Verification should shorten the path to working software. Broad commands run
while the implementation is still changing consume time and produce evidence
for code that is about to be replaced.

## Tier boundaries

| Tier | Timing | Purpose |
|---|---|---|
| **Tier 0** | At the edit boundary defined by the applicable stack rules | Compile/type/lint feedback for the changed slice |
| **Tier 1** | When the implementation unit is complete | Focused tests for the finished unit |
| **Tier 2** | At phase completion | Complete profile/suite integration for the phase |
| **Tier 3** | At milestone or release only | Release builds, supported-profile certification, E2E/visual, or other milestone evidence |

Rust rules permit related edits to form one cohesive implementation slice before
the consolidated check. During a failing repair, narrow to the focused failure;
do not restart an unchanged broad suite until the focused path passes. Do not
repeat an expensive command without a source change or a contract requirement.

## Delivery-first loop

1. Finish the smallest cohesive implementation unit in dependency order.
2. Use static inspection and formatting while editing.
3. Run the applicable Tier 0 once at its defined boundary.
4. Run the focused Tier 1 evidence after the unit and its tests are complete.
5. Continue implementation; reserve Tier 2 and Tier 3 for their actual boundary.

Tests are evidence and feedback, not the work queue. If a broad test reveals an
observed defect, repair the causal boundary and rerun the narrow failure first.

## Local means local

Unit, integration, conformance, lint, format, type, documentation, security,
load, soak, and release-candidate checks run locally. GitHub Actions are
**deployment-only**: they may build/package/deploy an artifact and validate the
resulting deployment. A hosted green check does not substitute for retained local
product evidence.

## Expensive and external boundaries

Real-model calls run only when the requirement claims inference and after the
code path is complete. Long-duration testing additionally needs a failure model,
traffic target, operating-period target, or statistical objective. If credentials,
capacity, model weights, network access, or budget are unavailable, report the
claim unverified; do not substitute a synthetic provider.

Every result remains bound to its source SHA, command, environment, and named
profile. Results transfer to no other profile by implication.
