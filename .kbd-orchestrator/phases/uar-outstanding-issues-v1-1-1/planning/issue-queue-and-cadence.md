# Repository issue queue and delivery cadence

Date: 2026-10-17. Active phase: `uar-outstanding-issues-v1-1-1`, Execute,
task 6.2. This is a planning disposition, not acceptance or an issue closure.
Current GitHub issue state was read on 2026-10-17; all issues below were open.
The active source is uncommitted. Preserve its checkout and failed receipts.

## Priority and release boundary

| Priority | Issues | Owning outcome | Current phase or successor |
| --- | --- | --- | --- |
| P0, start now | #385 | Restore actionable binding/durability/budget diagnostics and supported remote-Surreal/KB/team recovery guidance; prove the selected behavior on the candidate and both published children. Customer recovery remains a separate observation. | Current 1.1.1 phase; lead, runtime and state. |
| P0, same frozen candidate | #380 | Preserve truthful completion, MCP outcomes, Anthropic framing and credential authority through the existing six-group/four-supplement program. | Current 1.1.1 phase; runtime and trust. |
| P0 release blocker | #387 | Restrict both compiler REST aliases for remote or shared-credential profiles before publication; preserve local single-owner use and leave full owner-scoped authoring for a successor phase. | Current R11 production/doc/acceptance cohort; full ownership repair remains open. |
| P1, evidence handoff | #379, #383 | Reconcile delivered v1.1.0 criteria against immutable receipts without moving the old tag. | Current P3; lead. |
| P1, dependency tracking | #382, #386 | Keep Dot integration and media grants/effect receipts open with BossFang/KnowMe dependencies and no fabricated qualification. #386 excludes PSTN at this stage. | P3 dependency disposition; separately specified successor phase. |
| P1, next security/identity phase | #392 | Define trusted optional-JWT local identity across history, memory, KB and tools. It differs from #380 credential ownership. | Successor Spec/Plan; trust and state. |
| P1, context phase | #388 | Final per-destination context compilation and fallback fit. It differs from #385's conservative catalog reservation refusal. | Successor Spec/Plan; runtime and providers. |
| P1, Postgres correctness phase | #389, #390 | Explicit settings source provenance on restart and canonical owner-key TTL scanning. Neither is a #385 remote-Surreal repair. | Successor Spec/Plan; state and lead for settings. |
| P1, truthful storage phase | #393, #394, #395 | End silent Postgres memory-write acknowledgement; publish/implement supported durable capability parity and configured embedding dimensions. Do not claim unsupported profiles in 1.1.1. | Successor Spec/Plan; state. |
| P2, SDK release compatibility | #391 | Resolve the embedded Rust SDK exact path-version mismatch and verify an actual supported consumer if the SDK is included in a release claim. | Successor Spec/Plan, or current release gate if advertised; lead. |
| P2, protocol expansion | #396 | Add agent execution to an Anthropic-compatible mode without changing the current direct-provider `/v1/messages` claim. This is separate from #380 SSE framing. | Successor Spec/Plan; runtime and providers. |

The ten Dot architecture findings are not ten unobserved defects in the selected
1.1.1 acceptance program. Their source observations and issue bodies are inputs
to supported-profile decisions. A profile that promises a listed capability
must either accept it through its real production path or disclose and enforce
its absence. The uncomfortable case is #387: both `/api/uar/compiler` and
`/api/compiler` are mounted unconditionally, while compiler CRUD and session
storage do not carry an owner. JWT admission does not partition that storage.
Source inspection found this boundary; no live cross-tenant exploit has been run.
R11 therefore blocks the complete REST subtree whenever ingress is remote or
provider credential mode is shared, including reads, writes, reports, sessions,
inline compile and compile-and-register. The release must describe the profile
limit accurately. The owner-scoping work and closure of #387 remain separate.

Sources: [Dot candidate audit](/Users/gqadonis/Documents/Codex/2026-10-16/task-2/architecture-audit-main-91a091d.md),
[Dot issue packet](/Users/gqadonis/Documents/Codex/2026-10-16/task-2/uar-1.1.1-audit-issue-proposals.md),
`issue-385-coverage.md`, and the current GitHub issue bodies.

## Immediate #385 delivery path

1. Preserve the failed local04 receipt and classify the binding/recovery result,
   image identity discrepancy, cleanup failure and source supplements by actual
   cause. Record the current supported customer guidance separately from product
   acceptance. Do not mutate the know-me deployment under this phase plan.
   D1 found a host event-history bypass for the first binding refusal assertion;
   it also found that recovery and combined were blocked before UAR startup by
   Docker timeouts. These outcomes do not certify the other #385 rows.
2. Complete one observed repair cohort, with exact file ownership for runtime,
   trust, state and lead. Finish production, fixture, configuration and required
   documentation before a renewed whole-phase inventory. If source inspection
   shows no new #385 source defect, record that finding and keep its acceptance
   evidence pending rather than manufacture a repair.
3. Run only affected approved local gates after the inventory. Each #385 row in
   the coverage matrix needs a receipt or an explicit unverified disposition.
   Local candidate success precedes immutable tag/images and both published
   child-image programs. P3 distinguishes isolated repair from live customer
   recovery and does not close an unmet criterion.

## Team work packets and single writers

| Role | Next bounded packet | Exclusive source area after lead assignment | Handoff |
| --- | --- | --- | --- |
| `uar-lead` | Canonical issue disposition, #385 evidence matrix, runner/image provenance triage, shared roots, #387 route registration/release text, inventory, builds, publication. | `plan.md`, `execution-ledger.json`, `ownership.md`, OpenSpec records, `src/server.rs`, `docs/releases/v1.1.1.md`, `scripts/release/uar_1_1_1_acceptance.py`, `scripts/release/uar_1_1_1_support.py`, Dockerfile only if an observed correction requires it. | `coordination/handoffs/` with exact source and failed receipt links. |
| `uar-runtime` | Repair the confirmed binding event-history bypass; trace zero effective MCP policy in streams, and implement only confirmed runtime/fixture corrections. | `src/uar/runtime/manager.rs`, `src/uar/runtime/turn/host/event_projection.rs` only if needed, and `scripts/release/uar_1_1_1_runtime.py`; no lead shared files. | Cause, changed paths, interface effects, unverified cases. |
| `uar-trust-tools` | Source-inspect credential isolation block and migration/supplied-driver supplement; restrict #387 compiler REST with exact lead assignment. | Credential policy/API/fixture paths in `ownership.md`; `src/uar/api/compiler.rs` and the new compiler acceptance adapter only after explicit handoff. | Exact principal/backend observations and unverified cases. |
| `uar-state` | Source-inspect recovery/backend/supervisor and owned-resource failure; implement only confirmed state corrections. | Persistence/recovery adapter and supervisor paths in `ownership.md`. | Actual store/process/guest identities and unverified cases. |
| `uar-verifier` | Frozen artifact-only challenge at the next approved boundary, if a new review is explicitly added to the plan. | No production writer. | Findings with evidence; current two-round source review cap is exhausted. |

The lead assigns exact newly touched files before implementation. Other agents
share this checkout and preserve one another's edits. No teammate launches an
early check, test, build or external gate. Team coordination records do not
complete canonical KBD tasks. A fresh agent receives only the waypoint, relevant
issue/phase packet and exact owned paths, then returns changed files, decisions,
source state and what remains unverified.

## Calculated cadence

Use the installed `delivery-cadence` skill for a **120-minute work checkpoint**
and a **soft 7-day issue-sized delivery opportunity**. The first #385 checkpoint
begins immediately; the weekly opportunity is not a wait before blocker work.
At each work checkpoint, record the owned work span, source/handoff state,
remaining complete-phase implementation items, and whether a frozen candidate
exists. Do not turn a time boundary into permission for a test. Once a full
increment is implementation-complete, the lead freezes the candidate and runs
the planned real build, launch and feature operation serially. Keep publication
manual, with review for every successful delivery and one build writer.

The observed native dependency build was approximately 39 minutes and image
compilation 17 minutes 14 seconds: **56 minutes 14 seconds** of preparation.
Three serial six-group programs are required for 1.1.1 (candidate, published
arm64, published amd64). Failed local attempts lasted approximately 71, 156 and
306 minutes; none is a passing benchmark. Using 156 minutes only as a planning
scenario gives `39 + 17.23 + 3 × 156 = 524.23 minutes`, or about **8 hours 44
minutes after implementation**. Repeating the observed 306-minute failure tail
gives about **16 hours 14 minutes**. Publication, pulls, operator review, source
rework and release receipts have no measured duration and are excluded. A two-hour
release promise is therefore unsupported. Recompute the soft weekly target after
three accepted issue-sized deliveries using measured implementation, build,
operation and publication spans; never replace observed data with the scenario.

Planned Cadence profile: KBD binding to this phase while it remains active;
`iterationMinutes=10080` as a soft target, `reviewEvery=1`, manual publication,
one build writer, arm64 and amd64 acceptance obligations where the phase requires
them, and recommendation-only optimization after at least three accepted samples.
Record 120-minute work sessions as activities, not completed delivery iterations.
Do not initialize the profile until the exact build, launch and feature-operation
checkpoint commands and output roots are bound to this phase's approved real
runner; a new Cadence checkpoint must not duplicate an already passing gate.
`tick` evaluates an opportunity only when called; it installs no scheduler.
No automation, background agent or publication was created by this plan.

Apply skills by task: KBD Spec/Plan/Execute/Reflect and the relevant OpenSpec
change; `agent-team-manage` and handoff for ownership; Rust/security/provider or
state skills for the files being changed; `delivery-cadence` at work/delivery
boundaries. Do not run all installed skills on each clock tick. The full-phase
verification boundary and the six existing acceptance groups remain controlling.
