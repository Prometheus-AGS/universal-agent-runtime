# Dot feature support decisions

## 2026-10-11 — Analyze recommendations

- D-A01: Keep the current Rust/Tokio/Axum and SurrealDB/Postgres architecture. Provenance: repository constraints and inspected current source. No stack migration is needed to address the reported defects.
- D-A02: Extend existing host admission, observers, approvals, connector outcomes and team claims. Provenance: cand-001/cand-006. Kernel code must not acquire independent write authority. Counterevidence that would reopen this: existing transactions cannot represent required occurrence/settlement invariants without a second incompatible state model.
- D-A03: Conditionally adapt tokio-cron-scheduler 0.15.1 for timing only (cand-002). Gate: exact-version fit for required timezone/DST, cancellation and restart behavior; authoritative UAR records own due work. No installation or pin change approved here. Reject the adapter if it adds duplicated scheduling state without reducing recurrence complexity.
- D-A04: Keep Apalis and Temporal as references (cand-003/004). Their adoption is not justified by this bounded dual-backend analysis. Reopen for a deliberate hosted workflow platform requirement. Temporal is not rejected on the stale preview label returned by Context7; licensing and exact-tag API fit remain unresolved.
- D-A05: Treat versions.toml divergence as policy reconciliation work. Actual Liter input is the a6047386 path source at 2.1.1; memory is a committed vendor tree. This identifies source inputs without approving drift or certifying the image's transitive graph. Preserve exact-pin provenance in Plan.
- D-A06: Container publication succeeded for 2cc9a182, including both architecture manifests. Do not repeat landed release repairs; deployment health and stronger reproducibility evidence remain required.
- D-A07: Proposed schedule defaults (coalesce backlog, no overlap, explicit DST policy), notification destinations, memory retention and deployment priority remain proposals for Plan. The deployment-priority question is pending; local-first with shared-host-safe contracts is a provisional analysis assumption, not an operator choice.
- D-A08: Follow the operator's Assess → Analyze → Plan → Execute → Reflect sequence. The optional separate Spec stage is folded into Plan, where mandatory OpenSpec deltas still must be authored. No implementation tasks are registered by Analyze.

Decision scope: these are analysis recommendations, not product implementation authorization. Evidence and falsifiers are retained so Plan can revise choices without rewriting this history.

## 2026-10-11 — Operator resolves deployment priority

D-A09 supersedes D-A07's provisional local-first assumption and the deployment-priority question: the operator selected “Both equally from the first release.” Local operation and shared hosting are equal first-release requirements. Both must satisfy the six capability contracts, with separate deployment-profile evidence; shared-host work cannot be postponed as a later rollout. Backend and client-surface mappings remain explicit Plan decisions. Existing candidate verdicts remain recommendations, now constrained to preserve both profiles. Provenance: direct operator response to the deployment-priority question, not inferred preference.

D-A10 records the proposed support matrix added after clarification review: local SurrealKV and attested networked SurrealDB shared by multiple runtimes are first-release acceptance targets, not certified configurations. Plan must accept or replace the mappings and enumerate entry-point coverage. Postgres observer parity and direct embedded SDK coverage remain explicit blocking decisions for any corresponding support claim; existing functionality must be preserved. Rationale: the inspected Surreal provider advertises durable observers conditionally, while Postgres inherits the unsupported observer capability default. Neither observation proves runtime correctness. This is an analysis recommendation, not a new operator preference or authorization to remove a backend.

## 2026-10-11 — Plan boundary

- D-P01: Plan contains 17 vertical changes and 35 backend tasks. Local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes are equal first-release assistant acceptance profiles. Preserve and verify modified existing Postgres credential/key/accounting behavior; do not advertise absent observer/checkpoint parity or blanket direct embedded SDK support.
- D-P02: Keep existing UAR host governance, memory and persistence. tokio-cron-scheduler remains conditional on exact-version fit; no dependency is installed in Plan. First-release recurrence forms are once, daily and weekly, with explicit timezone/DST/revision rules; owner conversation inbox is the base follow-up destination and automatic capture is opt-in with explicit retention choice.
- D-P03: Complete owner identity uses a namespace disjoint from legacy subject keys. Migrate only proven ownership; preserve historical cost rows without inventing settlement identities. Existing active changes retain their task ownership and completion evidence.
- D-P04: Isolated review required a CAS continuation claim and exposed an ordering contradiction. Change 06 proves orderly fresh-process restore and denies abrupt takeover; change 11 supplies supervisor exclusion and proves successful abrupt-owner recovery for checkpoint and team continuations. Approval consumption remains current-authority and single-use.
- D-P05: Final artifact review PASS with two warnings carried as blocking prerequisites for affected tasks: supervisor adapter qualification before 11, and busy-time overlap semantics before 14. They require bounded plan/spec resolution, not silent implementation assumptions. Foundation changes with satisfied prerequisites can proceed. Exact producer identity is unavailable, so cross-model independence is unverified.
- D-P06: versions.toml explicitly prohibits agent edits. Change 01 must produce a concrete reconciliation for deliberate operator policy resolution before dependent source changes/certification. Existing amd64/arm64 publication is historical baseline evidence; final-revision publication and health follow local phase acceptance in 17.

No product code or runtime guard is added by this Plan. Model assignments are recommendations from the exposed Codex catalog, not worker-launch or provider-configuration changes. Next is Execute; neither implementation nor release is certified by artifact validation.

## 2026-10-11 — Operator resolves Plan questions

- D-P07 supersedes the pending supervisor selection in D-P05: the operator selected existing native service managers locally and Kubernetes for shared hosting. Existing native-service-deployment identifies macOS LaunchAgent, Linux systemd and Windows SCM; retain platform-specific evidence. Require authenticated incarnation-bound root/descendant exclusion; API Pod deletion or unreachable-node state cannot stand in for termination. Kubernetes documentation was checked through Context7 and cited in the recovery design. Real adapter qualification and infrastructure access remain Execute requirements.
- D-P08 supersedes the pending overlap policy in D-P05: coalesce to the latest pending occurrence while the previous run executes, awaits approval or has uncertain outcome. The guard spans stable owner/schedule identity across revisions and restarts; edits cancel old pending intent without bypassing exclusion. Safe release rechecks current revision, authority, budget and approval and atomically admits at most one pending occurrence. This records the operator choice, not a runtime result.

The two preference questions are answered. Existing task identities, ordering and model assignments stay unchanged; pin-policy resolution, scheduler fit and actual runtime qualification remain outstanding.

## 2026-10-11 — Execute operator resolutions

- D-E01 records direct operator approval to adopt main Liter 2.1.1/a6047386cc9fae4258b4a8577f6a016022095586 and skill gitlink f4bef4556a83ac22bf204bf69a01ddf0f9e007f2, including explicit authorization for the otherwise protected versions.toml edit. Memory upstream pins remain unchanged: exact comparison proved the source export matches the recorded commit except documented manifest adaptation. This resolves D-P06 policy permission, not migration, clean build or deployment acceptance. Prerequisite artifact review PASS; concrete judge identity unavailable, cross-model separation unverified.
- D-E02 resolves the retention warning in Plan clarification review: operator selected 7, 30, 90 or 365 days, or until-deleted. The proposed immutable first durable capture anchor, immediate shortening of existing automatic capture, extension only for new captures/no revival, and current-policy queued commit are retained. Exact 24-hour UTC boundary and interrupted-shortening/extension scenarios are added to change13 design/spec. Bounded independent artifact review and actual production acceptance remain pending. Existing explicit user memories are not silently reclassified.


## Final operator closeout — 2026-10-14

Only resource cleanup, consolidation on main, and phase closure/archive remain authorized. All further qualification, builds, deployment, reviews, and consumer-use gates are cancelled, without recording them as passed. See the Dot phase closeout record and decision dot-operator-closeout-20261014.
