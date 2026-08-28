# Reflection: allow-loopback-tools-without-jwt

## Delta

The plan called for a code freeze before certification and a single final quality gate. Delivery did not meet that sequence. The first isolated artifact critic found four material gaps after the initial certification: rollback ownership was documented incorrectly, status notifications could race runtime publication, direct HTTP tool execution still crossed Cedar while Governance was Off, and key live/recovery claims lacked durable receipts. After those corrections and recertification, a second isolated critic found that the HTTP bypass was broader than the requirement and that the retained recovery binaries were stale. The bypass was narrowed to `POST /api/tools/*/execute`, a non-tool regression was added, both forward and rollback candidates were rebuilt from their final commits, hashes and receipts were replaced, the LaunchAgent was updated again, and the complete local gate was rerun. A third isolated critic found stale KBD completion and publication claims, not a production-code defect; those records were corrected and the critic returned PASS before reflection. The final implementation therefore differs from the planned one-pass finish: it required two production corrections, two complete certification restarts, and a final state-record reconciliation.

## Root Cause

Authority was distributed across two execution boundaries. `RunManager` owned the documented approval flow, while the global HTTP Cedar middleware independently classified direct tool routes. The first implementation changed only the former, so the intended Off behavior was incomplete. The first HTTP correction then returned before route classification, which converted a tool-specific exception into a global authorization bypass.

Settings notification ownership was also split. The generic database notification could be emitted before the in-process runtime snapshot changed, so a remote panel could refetch stale authority. The implementation lacked a single explicit post-publication notification owner.

Rollback reasoning confused posture-derived initialization with operator-owned persisted state. An API-owned `false` row is intentionally preserved across rollback, so documentation that claimed normalization to `true` was false. Recovery records also identified a directory rather than commit-qualified artifact names, allowing superseded binaries to coexist with the final candidates.

KBD projections lagged the evidence. The plan said 42/42 before reflection, publication, verification, and archive were complete; progress also retained an obsolete certification blocker and a false no-PR conclusion. These were state-accounting failures, not implementation completion.

## Corrective Actions

The final runtime uses one coherent governance gate shared by direct HTTP middleware and tool execution. Governance Off bypasses Cedar only for `POST` tool-execution routes; non-tool actor and collaboration actions remain governed. Regression tests cover Off tool bypass, On tool enforcement, and Off non-tool enforcement.

The settings mutation sequence now makes the durable write, cache update, runtime snapshot publication, explicit governance notification scheduling, and response ordering visible. Implicit governance database notifications are suppressed, and post-commit delivery failure is logged without rolling back the accepted mutation.

Rollback tests and documentation now distinguish API-owned values from posture-owned defaults. Recovery uses commit-qualified forward and rollback binaries with independently verified SHA-256 hashes. Machine-readable receipts retain installed tool execution, non-tool denial, warning cardinality, release builds, and live status transitions.

The full Rust, frontend, browser, support-matrix, release-contract, OpenSpec, rollback, release-build, installation, and live behavior gates were rerun after the final production correction. Publication remains pending until this reflection is accepted, the affected branches are pushed, the forward PR exists, and OpenSpec is verified and archived.

## Remaining Risk

No third-party search MCP is installed in this runtime, so the exact search-provider path was established through deterministic tool-gate and direct HTTP execution coverage plus a configured native tool live receipt, not a live external web-search call. Existing PGlite eval warnings remain in the frontend production build and are unrelated to this change. Superseded unqualified recovery binaries remain in the retained backup directory, but the recovery procedure names only the commit-qualified final files and hashes. Non-loopback and JWT live receipts were captured on the immediately preceding forward source; the final source delta only narrowed the HTTP tool-route predicate, and the exact final source is covered by deterministic eligibility and middleware regressions.

## Independent Review

The isolated review sequence was FAIL, FAIL, PASS. The first two failures caused production and evidence corrections followed by complete certification restarts. The final PASS was returned after the production fixes, recovery artifacts, and KBD record corrections were present.

## Sycophancy Check

The specialized Reflect analyzer returned `sycophancy_score: 0.0`, `s08_detected: false`, and `No changes needed`. Its correction mode first failed server-side because the response omitted the required `corrected_artifact`; detection mode completed successfully and is retained in `sycophancy/reflect-analysis.json`.
