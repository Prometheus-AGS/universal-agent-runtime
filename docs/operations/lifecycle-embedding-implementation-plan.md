# Local embedding and immutable delivery continuation

Operator decision: 2026-10-20. Active phase:
`uar-outstanding-issues-v1-1-1::function-first-release-certification`, Execute.
Canonical registration: waypoint 3155, plan revision 70. Verification CLOSED.

The operator preserves the full embedding requirement. The earlier frozen
contract is not weakened: live primary-to-compatible-backup-to-primary recovery,
backup ingestion and query continuity, and actual incompatible-space refusal
remain required. Old mirror restart receipts do not satisfy live failover.
This authorization adds local-only implementation and fixture preparation;
it does not authorize hosted calls, new credentials, paid usage, personal
configuration mutation, or execution of tests and preparation commands.

## Implementation and ownership

| Canonical task | Outcome | Single writer |
| --- | --- | --- |
| 6.2 | Strict local primary/backup construction, compatible provenance, event-driven active generation, explicit recovery, retained inference workers and truthful authenticated status/control | Runtime: embeddings capability, embedding config type, embedding-only server/settings spans |
| 8.2 | Real local-model durable ingestion/query and incompatible-space fixture source, event/barrier-controlled lifecycle and cleanup | State: new embedding integration target and local fixture directory |
| 8.3 | Explicit candidate archive/immutable-source/protocol binding without relabeling public or other-platform artifacts | Boss-root worker: candidate integration and integrity loader; lead: UAR helper if needed and final immutable source revision |
| 9.3 | Scoped commits, push, draft PR, deployment-only SHA-addressed Docker image path, renewed source inventory and exact deferred commands | Lead |

No worker may run tests, checks, lint, formatting, typechecks, Cargo, compiler
probes, builds, packaging validation, policy validators or review gates. They
may inspect source and author production, fixtures and documentation. A model
asset absent from the declared local fixture must remain a failed prerequisite,
not a fake vector implementation, silent download or passing skip.

The root owns shared manifest integration, lifecycle dependency edges outside the
runtime-owned embedding spans, KBD/OpenSpec, Git and publication. Existing dirty
changes are preserved. Root tooling and unrelated global harness configuration
are outside the publication scope.

## Delivery order

1. Record the runtime API/config/control contract and share it with the fixture
   writer. Implement strict local members and compatible asset identity.
2. Author the real-model fixture against that production API. No execution.
3. Integrate acquired embedding scopes into the host dependency graph and
   collect every source handoff.
4. Commit the scoped UAR implementation; bind the Boss candidate to that actual
   immutable revision, then commit its scoped changes. No fabricated future SHA.
5. Update the existing UAR draft PR or create one if absent, and attach every
   task PR. Use the existing deployment-only image workflow on the exact pushed
   branch so candidate images are SHA-addressed rather than relabeling `main`.
   Do not add general verification to GitHub Actions or edit workflow-invoked
   scripts while the required policy validator remains closed.
6. Publish renewed inventories and exact later verification/preparation commands.
   Record a workflow request as requested, a running build as running, and image
   availability only after a real published digest is observed.

## Completion and uncomfortable limits

This continuation supersedes the previous source inventory until all production,
fixture, integration and documentation changes are collected. Source completion
cannot certify compilation, local inference, release of workers, live failover,
space rejection, Windows behavior or customer packaging. The operator's request
for a usable Docker build is a deployment deliverable, not permission to run
routine verification or certify the image. The existing image workflow has
build-only Vite/Cargo targets and deployment-specific manifest inspection.

Actual model initialization can fail or remain pending. The host must expose
that obligation; a timeout may not certify release. A local mirror is compatible
only through validated space/assets/inference identity. Matching dimensions
alone is insufficient. No hosted provider is silently substituted.
