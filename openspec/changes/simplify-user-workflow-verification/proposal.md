# Simplify UAR verification around independently runnable workflows

## Why
Current verification is repeatedly blocked by coupled fixture/authority preparation before genuine user workflows execute. A22 metadata projection failed before review/build. A basic configured prompt must not require every platform, store, SDK and embedding fixture.

## What changes
Introduce one coverage registry, workflow-scoped admission, a small serial result-collecting runner, lazy platform/store fixtures and generated compatible authority metadata. Preserve real inference oracles, all advertised capability coverage, existing failures, cumulative limits, owned cleanup and backend/platform honesty. Propose explicit child-specific gate policy changes for one bounded execution approval rather than repeated per-edit signature loops.

## Capabilities
- MODIFIED product-validation-evidence: independent workflow admission and complete mapped coverage with bounded failure collection.

## Impact
Planned scripts/acceptance runner/case/fixture adapters, verification documentation and explicitly approved local operator policy. No production source change, test execution, contract adoption or certification occurs while writing this proposal. Child uar-verification-simplification returns findings to parent uar-post-lifecycle-verification-readiness.
