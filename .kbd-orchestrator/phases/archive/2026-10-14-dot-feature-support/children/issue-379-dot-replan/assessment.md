# ASSESSMENT: dot-feature-support › issue-379-dot-replan

Project: Universal Agent Runtime
Date: 2026-10-12
Stage: Assess. This is source inspection and contract comparison, not runtime acceptance.
Codebase baseline: current shared checkout inspected without a clean-tree assertion; issue #379 describes a separate read-only audit of main at b4244aea.
Cross-tool progress: parent Dot progress records changes 01–06 complete, 07 in progress, and 08–17 pending. The parent waypoint reports 122/146 implementation tasks in the broader run.

## Decision supported by this assessment

Issue #379 is a real additional conformance request for Kubo Topology B, but the current Dot plan does not promise most of its protocol and configuration behavior. Preserve the six Dot first-release capabilities for local and shared hosting while planning issue #379 explicitly. Do not count the issue's source audit or the parent progress labels as a passing installed-artifact gate.

The uncomfortable thing: the parent plan and ledger put verification after each change, and even record completed verification tasks for 01–06, while the current operator rule prohibits any further verification before every Dot production implementation item is complete. A child replan can correct future order; it cannot retroactively turn those earlier receipts into full-phase acceptance. Expanding Dot with all nine Kubo items would also extend that boundary and risk both releases unless the operator makes a release-scope decision.

## Contract and source gap matrix

Status means source alignment to the issue's requested behavior, not a tested outcome. The issue and its linked conformance request define the nine items: https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379 and https://github.com/GQAdonis/kubo/blob/feat/ipfs-mcp-server/mcp/uar/requests/CONFORMANCE.md.

| # | Assessment | Dot overlap and source evidence | Missing contract / dependency |
|---|---|---|---|
| 1. A2A 1.0 card | MISSING | Dot 16 promises assistant operation descriptors, not an A2A card. The well-known router only registers /agent.json (src/uar/api/a2a/mod.rs:74–76); AgentCard retains the older URL and snake-case fields (types.rs:262–279). | Public /.well-known/agent-card.json, 1.0 schema and interface URL. Source mount passes through server routing/auth layers (src/server.rs:1612, 2034–2049); public access must be specified and inspected end to end. |
| 2. A2A 1.0 task state | MISSING | Dot 07 recovers approvals, but does not alter A2A wire values. TaskState uses lowercase serialization and lacks AuthRequired/Rejected (src/uar/api/a2a/types.rs:117–132); gRPC maps InputRequired to working (grpc.rs:369). | A2A 1.0 state, role, part, and method contract, including insufficient-scope behavior. Precedes the issue's A2A approval acceptance. |
| 3. A2A approval | PARTIAL foundation | Dot 07 covers durable pending decisions and safe continuation; Dot 11 covers dead-owner exclusion; Dot 16 exposes approval reads/decisions. A2A task construction still sets ToolApprovalPolicy::Auto (src/uar/api/a2a/task_execution.rs:568); manager has a governance-disabled bypass branch (src/uar/runtime/manager.rs:6175–6190). | A2A task must expose input-required with approval ID, await decision, and never bypass a required tool under either governance setting. Policy semantics need an explicit decision before implementation. |
| 4. AG-UI interrupts | PARTIAL foundation | Dot 07 supplies approval state and Dot 16 promises consumer APIs. Current adapter emits approval as CUSTOM uar.tool.approval_required (src/uar/api/adapters.rs:363–381); no RunAgentInput reference was found in the searched src/uar/api surface. | Documented RunAgentInput endpoint, spec SSE events, RUN_FINISHED interrupt outcome and resume handling. Needs the same durable decision authority as 07. |
| 5. CORS allowlist | MISSING | No Dot change owns CORS. Server builds an Any-origin/method/header layer (src/server.rs:2609–2614). | Configured origin list with empty default and denial of unlisted origins. |
| 6. MCP elicitation | MISSING | Dot 04 owns MCP outcome truth, not client elicitation. Client connections use the unit handler in src/mcp/registry.rs:698,779,834,1400; no elicitation/create handler was found in the searched client path. | Advertise elicitation and route server requests through an interactive A2A or AG-UI channel, then return accept/decline/cancel; depends on an interrupt path and durable approval/continuation behavior. |
| 7. Safe defaults | PARTIAL | Dot 03 makes API keys durable but does not set these defaults. Config sets jwt_required true, server.host 0.0.0.0, remote memory embedder and openai/gpt-4o model (src/config.rs:1529–1530, 1580–1594); config.embedded.yaml:18–19 disables JWT. | No implicit model, local/off memory embedder, loopback bind, authentication in every shipped profile. Installed-artifact egress and socket checks remain unrun. |
| 8. Effective settings | PARTIAL source data | Dot 16 capability discovery does not promise effective configuration. GET settings returns per-key records and uses x-uar-admin-key (src/uar/api/settings.rs:75–78, 363–376, 616–633). | Authenticated read-only aggregate with provider hosts, governance, shell tools, MCP servers and change indicator; Bearer admin 200, missing 401, non-admin 403. Resolve schema wording before planning. |
| 9. Card release version | MISSING | Dot 01 handles locked inputs and Dot 17 final provenance, but neither promises card metadata. Card hard-codes 0.1.0 (src/uar/api/a2a/agent_card.rs:22). | Built release version in card, tied to item 1 and final tagged published artifact. Do not assume the issue's suggested v1.1.0 tag is approved. |

## Dot alignment and dependencies

The parent plan's 17 changes remain: 01–06 source/identity/keys/tool outcomes/streams/resume; 07 approval recovery; 08 ingestion; 09–10 settlement and budgets; 11 supervisor fencing; 12–13 memory; 14 schedules; 15 follow-ups; 16 consumer capability surface; 17 local/shared acceptance and publication. Only 03, 04, 07, 11, 16 and 01/17 provide foundations or shared files for #379; no other Dot change claims the nine wire/configuration requirements. The plan's own dependency graph puts 11 after 07 and 10, 14 after 11, 15 after 13 and 14, 16 after 15, and 17 after all 01–16 (plan.md:186–274).

Kubo's issue names items 1, 8 and 9 as its check-uar minimum. Its full gate requires all nine in a published tag plus OCI digest or package checksum, tested from that artifact by Kubo's deadline. The linked conformance request permits a documented alternate settings shape with Kubo translation, whereas the issue describes its exact schema as check-uar input. This needs one pinned acceptance contract. A2A approval with governance disabled is another explicit policy conflict to settle, not an incidental mapper fix.

## Cross-tool progress and verification boundary

Parent progress.json: 01–06 DONE/COMPLETE; 07 IN_PROGRESS with implementation task recorded done and verification task pending; 08–17 PENDING. The waypoint currently points to this child Assess phase. It reports 122/146 broader run implementation tasks; that counter is not 122/146 Dot changes.

Build health: UNKNOWN. Test coverage and installed-artifact conformance: UNKNOWN. No test, build, lint, typecheck, policy validator, compiler probe, release gate or runtime request was run for this assessment. No parent plan, progress ledger, source or spec was edited. The required full-Dot implementation-complete inventory has not been recorded; no new verification is authorized by this artifact.

## Open decisions and unverified claims

Decide whether #379 ships in the first Dot release, in a later separately tagged release, or through a formally expanded parent phase. Decide the exact effective-settings schema/path, A2A approval semantics when governance is disabled, and release version/deadline. Source inspection does not prove Kubo's runtime acceptance checks, public-card accessibility, scope behavior, no outbound traffic, or cross-process approval behavior. The child Analyze/Plan stages must specify file ownership and an implementation-first verification schedule without altering the parent Execute ledger prematurely.

ASSESSMENT COMPLETE


## Architecture audit correction — 2026-10-12

The operator requires all nine issue #379 requirements in the first Dot release. Separate-release alternatives above are superseded. The source claim that A2A production sets ToolApprovalPolicy::Auto at task_execution.rs:568 is withdrawn: that line is inside the test module declared at 422–423. The old TaskStore is not the production handler store; A2AThreadService uses persisted actor threads, with process-local correlation and incomplete pending-interaction projection. See analysis.md for corrected evidence. The governance-bypass precedence finding remains independently supported in manager.rs:6147–6182. MCP elicitation is a distinct interaction and its acceptance must not itself grant tool authorization. Current memory defaults select a remote provider but memory is disabled by default; selection alone does not prove outbound traffic. Runtime behavior remains unverified.
