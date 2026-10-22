# Caller tools: bounded functional exit contract

Date: 2026-10-22. Project: universal-agent-runtime.
Active phase: uar-post-lifecycle-verification-readiness; existing change: openai-client-tools.
This is a KBD Plan amendment during Execute, requested by the operator. It does not create another phase or capability. It applies the installed kbd-plan protocol and the independent client-tools-exit-contract-recommendation.md. It preserves the existing baseline phase obligations and operator verification lock outside the authorized local tool-test/fix/retest/publication loop.

## Function being delivered

A caller can declare functions, receive calls, execute them outside UAR, submit matching results, and continue the conversation through OpenAI Chat Completions, Anthropic Messages, AG-UI, A2A and ACP. Existing host tools continue through their existing governed execution path.

For agent teams, the same function works through the installed orchestrator-agent graph's local AgentNode delegation. Continue the same selected worker role and graph node under a fresh governed child invocation; do not promise the same physical child process/thread. Preserve its canonical history, graph position and cumulative budget. Fresh roots obtain their own required spawn approval. A completed identical retry creates no new root, approval, child, provider call or host effect.

## Required evidence: finite, already-authored scenarios

| ID | Required result | Existing evidence path |
|---|---|---|
| CT-1 | All five native interfaces return caller calls and accept complete results; JSON/SSE where supported, full history, multi-round calls, choice modes, parallel controls, mixed host/client batches and text-only regression pass. | Existing 95-case matrix. |
| CT-2 | Caller schemas never grant host execution; collisions, malformed declarations, forged/missing/duplicate results, changed history/authority, cross-owner/tenant access and withdrawn executable bindings fail without forbidden effects. Provider failures and cancellation never become successful completion. | Existing 60 safety and five explicit cancellation cases; core negative/retry cases. |
| CT-3 | Ordinary cold continuation and exact completed replay preserve protected history and do not repeat provider/host effects; concurrent/conflicting claims fail closed. | Existing ordinary restart seed/resume and core concurrency cases. |
| CT-4 | The actual built-in local team graph passes scoped spawn approval, child caller handoff, continuation and cold recovery on all five interfaces. Observe router, worker, worker without router replay; actual root/child correlation and parent/child cleanup succeed. | Existing graph warm and graph restart seed/resume modes. Public redaction is preserved; hidden spawn arguments are not claimed verified. |
| CT-5 | Cumulative budget remains enforced warm and after restart on all five interfaces; settled mixed host effects/history survive abrupt candidate crash, with no repeat on continuation/replay. | Already-authored budget warm/cold and mixed-crash seed/resume modes. These promises are bounded to acknowledged handoff, not arbitrary crash-mid-effect exactly-once execution. |
| CT-6 | Owned runtime/fixture/request processes actually join. Unexpected forced kill, lost observation or unconfirmed cleanup fails the relevant gate. The single intentional crash is separately recorded and never called graceful cleanup. | Existing supervisor lifecycle receipts. |
| CT-7 | Publish the repaired immutable patch source and paired Docker tags, then pass the scoped local functional path against its exact image digest. Notes/docs identify source, digest, backend and evidence limits. | Existing test-disabled deploy workflow, registry/release receipts, local digest-bound container gate. |

The selected local acceptance backend is SurrealKV with the real production HTTP/runtime/provider paths and an isolated controlled LLM/MCP HTTP peer. This is not paid-provider certification or a PostgreSQL functional matrix. Native compilation includes both persistence implementations. Published-image runtime evidence will identify its actual platform; building arm64 is not arm64 functional acceptance. The image gate certifies the server tool paths and container userspace under isolated acceptance configuration, not every bundled UI/model/skill asset.

## Ordered remaining work

1. Acceptance writer finishes existing task13.23: source-defined redacted approval display, exact owned root/challenge, controlled router selection, observed child/worker identity, native AG-UI/A2A approval continuation and owned HTTP cleanup. Resolve demonstrated review findings only. Complexity S; owner Codex acceptance teammate.
2. Lead reconciles canonical tasks, OpenSpec and exact source inventory. Correct the stale OpenSpec13.17 authoring checkbox from canonical completion and actual safety receipts. No new suite is created. Complexity S.
3. Single gate owner completes CT-1 through CT-6 with the existing supervisor. Build only after production source changes; fixture/docs-only repair does not invalidate build012's production bytes. Complexity M, uncertain until the first actual post-approval graph run.
4. Release owner publishes the separately versioned repair (currently unpublished1.2.2), records source/index/platform identities, and performs CT-7 locally. Existing1.2.0 and1.2.1 source/image tags stay fixed. No product tests run in GitHub Actions. Complexity M.
5. Record the result and stop. Do not reopen unrelated phase certification. Complexity S.

## Commands and proof reuse

The existing full owned local gate is:

`python3 scripts/release/client_tools_local_gate.py --evidence target/client-tools-local-gate-012 --extended`

Use a fresh evidence directory on later invocations. A necessary production rebuild is exactly:

`CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features server-full,postgres-backend --bin universal-agent-runtime`

These remain within the operator's existing local functional repair authorization; the plan itself grants no unrelated check or publication permission. Publication retains required enabled hooks. Exact Docker command/digest and its minimal fixture bundle are recorded before execution after publication.

Preserve candidate-bound passing evidence when production/configuration and applicable assertions are unchanged. Retest failed/affected modes when the existing supported lifecycle permits it. The supervisor currently runs its matrix before extended modes; use it rather than invent flags or build a new resume framework. Do not require repeated full runs merely because a test-only assertion was corrected. One native acceptance run and one delivered-image run serve different identities and are justified; routine duplicate broad verification is not.

## Scope freeze, stop rules and current truth

No new features, refactors, providers, remote-team federation, arbitrary graph engine certification, recursive-team features, generic exactly-once recovery, performance/soak campaigns, UI/toolchain work, all-backend matrices or historical estate certification are blockers. No standalone verification infrastructure or additional test cross-products are added to reach this exit. Keep their limitations explicit rather than calling them passed.

Success exit: every CT row has passing evidence for the delivered source/image, required docs and immutable publication receipts are delivered, and cleanup succeeds. Stop feature work immediately then.

No-progress exit: after two failed repair/retest cycles of the same scenario without a new causal discriminator, or60 active minutes without a new required passing scenario or a concretely narrowed defect, stop automatic repair and report the exact failure, source identity and smallest remaining decision. Build/deployment wait time is reported separately. This never waives a failing requirement or labels a blocked result complete. A genuinely new capability requires a separate operator scope decision; a defect violating a CT row is repaired within this scope.

Current evidence: build012 passed compilation. Gate011 passed95 core,60 safety,five cancellation and ordinary cold continuation; all graph cases refused the redacted approval before child execution. Graph warm/cold, budget warm/cold, mixed crash and published repair-image acceptance remain pending. Existing1.2.1 was published before acceptance;1.2.2 is not published. No duration guarantee or whole-product certification is made.

## Current execution receipt, 2026-10-22T23:44Z

The historical Current evidence paragraph above is superseded by nativegate017 anddiagnosticgate018: all9scopedgroupspassed, actualownedcleanupconfirmed. Immutable1.2.2 publishedsourcec065/index048d9c8e thenfailedexactLinuxamd64 image001 onAGUIhandoffsetup(94/95core); extendedimagegatesnotrun. Newsource61/61 completeatrevision265/plan96 includes reviewedper-runacknowledgmentserialization, fixedstagediagnostics andversion1.2.3/contracts. Build017active, nativegate019 next, thenimmutable1.2.3publication/exactimagegate002. Originalinnererrorunproven; noacceptancewaiverornewscope. Earlier1.2.0/1.2.1/1.2.2 tags remainfixed.
