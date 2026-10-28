# Published-source UAR1.3.0 contract refresh — 2026-10-24

Phase `uar-post-lifecycle-verification-readiness`, Execute. Product verification **CLOSED**. Source inspection and authorized owned document authoring only. Shared checkout preserved; no canonical projection/journal, signer trust, credential, source implementation or other writer's file was mutated. No test, lint, formatter, compiler, build, hash/signature/schema check, review-driven verification, service/registry probe, script gate or Cargo command ran. No verification tier or product acceptance credit is claimed.

## Owned delivery

Changed only:

- `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/replacement-exit-contract.v1.3.0.proposed.json`
- `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-exit-contract.md`
- This handoff.

JSON fields changed: `adoptionSource`, `amendmentPolicy`, `releaseClaim.currentCapabilities`, new `releaseClaim.publishedCandidate`, `releaseClaim.explicitUnknowns`, BTHREE-01 `oracle` authority wording, `budgets.tokenFallback`, `budgets.consumedAtAuthoring.source`, `boundaryPacket.requiredApprovals`, `status.leadReportedCanonicalPosition`, `status.implementationCompleteMeaning`, `status.recommendedBthreePolicy.state`, and `status.proposedRemainingAllowance.{state,canonicalHighWaterAtPreparation.source,policyChange}`. Scenario IDs, production commands, runtime oracles/negative controls, numeric budget, exclusions, safety requirements and finite obligations are preserved. BTHREE wording acknowledges approved A policy but still requires actual signed adoption. No runtime requirement was waived.

Source binding now names production source `774469746fee4de99be41dc1cd231857067e4487`, tag `v1.3.0`, deployment40091743841, index `sha256:f67a36f1a6fb886379c6947f605a88945a94481c8b3c4e1d998b9b481cea0697`, amd64 runtime digest `sha256:20c58178cadb449acac00a88b72b33b5ab759e24375236ee5313e77f2d267506` and arm64 runtime digest `sha256:3006fbfaf2fb9937663f6de49b62615af75fe79e0a4e6f8b20f42cd48bf5ff4b`. These are attributed to retained `evidence/publication-v1.3.0-004/v1.3.0-publication-004-receipt.json`, not a fresh probe. Publication is successful and product acceptance pending. ARM publication does not expand the finite supported-profile/runtime scope.

Readable checklist now removes stale active/failed-publication claims, preserves all five exit categories and states the exact remaining authority sequence. It explicitly retains BTHREE disposition PASS versus runtime NOT_EXERCISED, COUNT trusted-builder-only versus standalone-main UNSUPPORTED, and all current-candidate acceptance obligations.

## Position and authority

Read waypoint revision324/plan107, active task update-parking-lot-admission/5.1, Execute/IN_PROGRESS, run-wide144/167. Prior whole-phase inventory/ledger still contain older revisions/publication states; do not use them as final readiness. Lead must reconcile them with complete task ledger and all writers' handoffs and bind final local contract/fixture/packet bytes separately from published production source. `implementationComplete:true` remains the existing scoped authoring claim; updated meaning explicitly denies canonical completion or gate readiness by that boolean alone.

A policy authority is retained, not re-requested: finite BTHREE historical disposition and a new verification-only epoch at240 active minutes/240 calls/2 repairs/3 no-progress checkpoints/no new spend. Historical effort remains UNKNOWN, canonical floors remain binding, new epoch NOT_STARTED until actual signed adoption AND exact F/V gate approval. Earlier empty floor map is marked a historical lead observation, not a fresh query.

Lead next: generate actual amendment bytes using the exact previously approved template; present bytes/digest/scenario diff; obtain actual operator signature; perform only explicitly approved trust registration and exact adoption. Previous digest `dab4675cd1d2889fbe627cf130ff2875ac2b2ab8c0dde1dddafb9042a648d0cb`, supersedes `contract-adoption-uar-post-lifecycle-verification-readiness-dab4675c`. This author generated no amendment, signed nothing, registered no trust and adopted nothing. Never replay the historical signature or claim historical scenario equality from hashes alone.

After actual adoption, obtain separate exact F lifecycle/effects/cleanup and V serial command/review approval against renewed inventory and adopted digest. No additional preflight is implicit. First failure stops; an implementation correction closes permission and requires renewed inventory/instruction. No repeated green-seeking run, broad campaign or successor.

## Commands and evidence limits

Executed only `cat`, `rg`, `sed` source/document reads; Python JSON composition and shell heredocs authored owned documents and exited0. Read installed KBD orchestrator instructions; its automatic mutating refresh was not executed under the source-only assignment and CLOSED lock. Read AGENTS lock/team instructions, prior recovery/replacement handoffs, waypoint, ledger/inventory and publication/approval receipts. No accepted result was invented.

Remaining unverified: actual JSON/schema/helper acceptance; normalized contract/scenario digests; exact scenario diff; trust/signature validity; signed adoption; final whole-phase reconciliation and effective local packet identity; current canonical consumption floors; all required product/runtime/profile/fixture/cleanup/review/spec/closure evidence. All original finite obligations remain. No production guards or unrequested capability were added. Documented guards trace to existing owner/identity/effect/cleanup trust boundaries and explicit operator lock, not speculative code.
