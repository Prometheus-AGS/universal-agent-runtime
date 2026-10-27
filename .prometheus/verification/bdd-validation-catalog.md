# UAR BDD validation catalog — proposed baseline v1

Prepared 2026-10-27 during `uar-verification-simplification`. This is the concrete scenario design requested by the operator. It authorizes no execution and creates no new runner. Adopt or trim this baseline explicitly before calling it the release requirement. Current manual acceptance stays one scenario at a time, Mac → Linux → applicable iOS.

## Counts you can use

| Priority | Concrete scenario definitions | Existing Mac outcome credit | Remaining on Mac |
|---|---:|---:|---:|
| Common user workflows, C01–C54 | 54 | 33 | **21** |
| Additional configuration and resilience, E01–E28 | 28 | 0 | 28 |
| Remaining public client/mode contracts, I01–I20 | 20 | 0 | 20 |
| **Full proposed baseline** | **102** | **33** | **69** |

Updated 2026-10-27 after C52 passed ([receipt](c52-startup-retention-macos-pass.json)); earlier: C53 passed ([receipt](c53-browser-agent-selection-macos-pass.json)); earlier: C35 passed ([receipt](c35-browser-history-reload-macos-pass.json)); earlier: C54 passed ([receipt](c54-team-recovery-macos-pass.json)); earlier: C36 passed ([receipt](c36-team-follow-up-macos-pass.json)); earlier: C43 passed ([receipt](c43-skill-reattachment-macos-pass.json)); earlier: C49 passed ([receipt](c49-regeneration-resume-macos-pass.json)); earlier: C48 passed ([receipt](c48-embedding-same-dimension-macos-pass.json)); earlier: C42 passed ([receipt](c42-skill-scope-policy-macos-pass.json)); earlier: C46 passed ([receipt](c46-request-overflow-macos-pass.json)); earlier: C45 passed ([receipt](c45-sliding-window-macos-pass.json)); earlier: C44 passed ([receipt](c44-truncate-middle-macos-pass.json)); earlier: C47 passed ([receipt](c47-kb-source-replace-macos-pass.json)); earlier: C40 passed ([receipt](c40-skill-reload-macos-pass.json)); earlier: C41 passed ([receipt](c41-skill-attachment-macos-pass.json)); earlier: C51 passed ([receipt](c51-provider-failure-macos-pass.json)); earlier: C50 passed ([receipt](c50-run-cancellation-macos-pass.json)); earlier: C39 passed ([receipt](c39-memory-lifecycle-macos-pass.json)); earlier: C38 passed ([receipt](c38-mcp-tool-approval-macos-pass.json)); earlier: C37 passed ([receipt](c37-native-tool-approval-macos-pass.json)); earlier: C34 passed on Mac (attempt 5, [receipt](c34-live-roster-routing-macos-pass.json)), following C33 (attempt 1, [receipt](c33-rest-model-restart-macos-pass.json)): there are **53 passing BDD scenario reports plus one standalone prompt receipt**, not 54 clean BDD feature passes. C03 is composite persistence/query evidence; its failed combined run remains failed. C05 holds four existing B labels in one scenario. The bounded behavior labels are now 57 (B36 from C33, B37 from C34, B38 from C37, B39 from C38, B40 from C39, B41 from C50, B42 from C51, B43 from C41, B44 from C40, B45 from C47, B46 from C44, B47 from C45, B48 from C46, B49 from C42, B50 from C48, B51 from C49, B52 from C43, B53 from C36, B54 from C54, B55 from C35, B56 from C53, B57 from C52). The 48 remaining definitions are planned, not 48 known failures, and do not need to run together. Start with the 0 common missing workflows. Extended methods/clients stay visible so a smaller core pass cannot be called full support.

One ID means one concrete Given/When/Then scenario. No Scenario Outline expansion, implicit three repetitions, default/custom cross-product or unlisted examples are hidden in these counts. Some scenarios have several steps because their user goal is a lifecycle, governed action or recovery. All stated outcomes must hold for that ID to pass. If implementation splits such an ID into independently run cases, revise this denominator before running them.

## Why this shape

The operator already selected small Node.js/Cucumber scripts, real configured inference, persistent useful agents, shared running UAR, and stop-at-PASS. Rebuilding a global fixture/registry/contract engine would repeat the failure that prompted this redesign. Running only a generic prompt through every feature would miss real defects in persistence, retrieval, context and team joins. This proposal uses distinct observable product outcomes and names extra configured choices separately.

The uncomfortable limit: broad support costs more cases than a prompt smoke test. This proposal has 102 specific behavior/client definitions even after removing the old default/custom multiplication. It is a **proposed product-workflow baseline**, not proof of reliability under all timings, model combinations or loads, and not full GUI-admin/release/security certification. Reduce it by explicitly changing a support claim or deferring a row, never by silently hiding variants or calling them passed.

## Preserve the work already accepted

All rows below have **COVERED — historical Mac** status. They are not automatically scheduled again. Read the linked B-row and receipt for exact source, model, store and limits. C18 is final budget fitting, not structural truncate_middle. C20/C21 show none-strategy context unchanged during restart. C19 shows real summarize. No pass transfers to Linux, iOS or PostgreSQL.

| ID | Accepted user outcome | Existing behavior/evidence |
|---|---|---|
| C01 | Default-agent prompt | B01: [prompt](2026-10-26T06-38-40.440Z-prompt-macos.json) |
| C02 | Incremental useful answer | B02: [stream](2026-10-26T06-56-36.601Z-streaming-macos.json) |
| C03 | Saved custom agent across restart, then prompt | B03: [math](agents/math-specialist-db64c2a8.verified.json) |
| C04 | Writing intent selects saved writer | B04: [writer](agents/writing-specialist.verified.json) |
| C05 | Create, index, bind and use knowledge | B05: [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json); B06: [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json); B07: [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json); B08: [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| C06 | Concurrent specialists produce a merged article | B09: [TEAM-01 PASS](team-01-macos-pass.json) |
| C07 | Conversation follow-up remembers earlier exchange | B10: [CONTINUITY-01 PASS](continuity-01-macos-pass.json) |
| C08 | Default and orchestrator deletion is refused | B11: [PROTECTED-AGENTS-01 PASS](protected-agents-01-macos-pass.json) |
| C09 | Update custom instructions, observe effect, delete agent | B12: [AGENT-LIFECYCLE-01 PASS](agent-lifecycle-01-macos-pass.json) |
| C10 | Anthropic Messages prompt | B13: [ANTHROPIC-01 PASS](anthropic-01-macos-pass.json) |
| C11 | A2A v1 completed task | B14: [A2A-01 PASS](a2a-01-macos-pass.json) |
| C12 | ACP streamed completion | B15: [ACP-01 PASS](acp-01-macos-pass.json) |
| C13 | Dedicated AG-UI streamed answer | B16: [AGUI-01 PASS](agui-01-macos-pass.json) |
| C14 | A2UI response over chat agui_spec | B17: [A2UI-01 PASS](a2ui-01-macos-pass.json) |
| C15 | Browser renders A2UI and action reaches agent | B18: [A2UI-BROWSER-01 PASS](a2ui-browser-01-macos-pass.json) |
| C16 | Keyword skill selection and use | B19: [SKILL-DETECTION-01 PASS](skill-detection-01-macos-pass.json) |
| C17 | Paraphrased local-embedding skill selection and use | B20: [SEMANTIC-SKILL-01 PASS](semantic-skill-01-macos-pass.json) |
| C18 | Final token-budget fitting with fact recall | B21: [CONTEXT-BUDGET-01 PASS](context-budget-01-macos-pass.json) |
| C19 | Actual conversation summarization preserves facts | B22: [CONVERSATION-SUMMARY-01 PASS](conversation-summary-01-macos-pass.json) |
| C20 | Conversation survives graceful restart | B23: [CONVERSATION-RESTART-01 PASS](conversation-restart-01-macos-pass.json) |
| C21 | Concurrent same-conversation turns survive restart | B24: [CONCURRENT-CONVERSATION-RESTART-01 PASS](concurrent-conversation-restart-01-macos-pass.json) |
| C22 | REST user preference survives restart | B25: [REST-SETTING-RESTART-01 PASS](rest-setting-restart-01-macos-pass.json) |
| C23 | Retained agent KB remains usable after restart | B26: [KNOWLEDGE-RESTART-01 PASS](knowledge-restart-01-macos-pass.json) |
| C24 | Different-dimension embedding regeneration | B27: [KNOWLEDGE-REGENERATION-01 PASS](knowledge-regeneration-01-macos-pass.json) |
| C25 | Document deletion removes its retrieval hits | B28: [KNOWLEDGE-DOCUMENT-DELETE-01 PASS](knowledge-document-delete-01-macos-pass.json) |
| C26 | Deleted linked KB does not break ordinary answering | B29: [LINKED-KB-DELETE-01 PASS](linked-kb-delete-01-macos-pass.json) |
| C27 | Unlink KB while preserving its source | B30: [KB-UNLINK-01 PASS](kb-unlink-01-macos-pass.json) |
| C28 | Relink retained KB and restore use | B31: [KB-RELINK-01 PASS](kb-relink-01-macos-pass.json) |
| C29 | Answer combines two linked KBs | B32: [TWO-KB-ANSWER-01 PASS](two-kb-answer-01-macos-pass.json) |
| C30 | Unlink one KB and use only the other | B33: [KB-UNLINK-ONE-01 PASS](kb-unlink-one-01-macos-pass.json) |
| C31 | Disable KB access and retain links | B34: [KB-DISABLE-01 PASS](kb-disable-01-macos-pass.json) |
| C32 | Re-enable KB access and reuse links | B35: [KB-REENABLE-01 PASS](kb-reenable-01-macos-pass.json) |

## Fixture vocabulary and ownership

Every future scenario has one scenario author/triager as owner. The operator chooses the next ID; the author owns its feature/config/steps and result logging. The external host operator owns already configured services, models, keys and identity ingress. The relevant capability maintainer owns a demonstrated production fix. Supporting setup is a prerequisite, not an extra passed behavior.

| Code | Minimum precondition |
|---|---|
| A | Running UAR, already configured genuine model, nonsecret base URL/workspace and saved agents as needed. |
| R | Owned runtime restart adapter for the selected OS, retained disk store and captured original settings. Restart only that owned instance; leave it running afterward. |
| B | Shipped UI, actual browser and browser BDD support. Record where both browser and UAR run. |
| T | Owned team, saved math/writer definitions, observable task/thread/run/artifact identities. |
| N | Small actual native tool with an inspectable effect/result, owned paths only. |
| M | Actual owned MCP peer provisioned externally, exposing deterministic private result/effect logs. It is real tool plumbing; it does not fake a positive model completion. |
| D | Owned durable store and actual memory/settings surfaces, disk-backed Surreal first; same outcome contract on PostgreSQL later. |
| F | Tiny disposable standard skills with private rules/markers plus a decoy; method/policy observation from the actual run. |
| C | Fresh fact-bearing multi-turn history, selected explicit structural strategy, source/prepared/final receipts and effective input/output allowances. |
| K | Disposable tiny corpus/KB and fresh private facts absent from the prompt, agent instructions and prior session. |
| E | Genuine embedding bindings provisioned outside the test. Use local where suitable; same-dimension case needs two different actual fingerprints at the same width. |
| X | A named local controlled fault with an owned target, exact trigger and inspectable state; no shared service/network disruption. |

No script requires, reads, creates or sends JWTs, bearer/API authentication keys or login credentials. Two-owner cases take two externally provisioned base URLs, not auth headers. Provider credentials remain service configuration outside tests. Preserve separate **$10 Cohere / $10 OpenAI embedding caps**, prior spend and all recorded history. Positive answers use genuine configured inference; controlled faults are only for negative/recovery cases.

## Common unfinished workflows — do these first

C33 and C34 **PASSED on Mac 2026-10-27** (see their entries). All C35–C54 are **PLANNED / NOT RUN**. Order is a proposal based on anticipated usage; execution remains operator-selected, one ID at a time. Shared setup may be reused without rerunning an earlier acceptance script.

### C33 — REST model configuration is effective and durable

- **Given:** an owned provider configuration and two already provisioned genuine model bindings A and B.
- **When:** save B as the REST default and a bounded output setting, save A as one session override, refuse an invalid revision/override, restart the owned UAR and prompt both sessions without a request override.
- **Then:** actual dispatch uses B for the new default session and A for the saved override; the output limit applies and each rejected write leaves the valid saved state unchanged.
- **Prerequisites/effects:** R, two existing model bindings; credential fields omitted; public setting/provider-default/session-agent-config operations from cases_agents.py effective_configuration. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** redacted before/after settings, rejected write/readback, restart identity and actual effective dispatch/usage; restore original settings. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U20.effective-configuration`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27, attempt 1:** `scripts/behaviors/rest-model-restart.cucumber.mjs`, 1 scenario / 11 steps passed in 2m 4.7s. A = `liter/MiniMax-M3` (session override), B = `liter/k3` (saved default, output limit 32768 → 1024). Requests naming no model ran on B/A with correct marked sums before and after restart (PID 21601 → 69961, same binary/config/store); B's provider-reported output was exactly 1024 tokens against a 4096 request both times. Stale revision → 409, invalid model → 400, saved state unchanged. Original configuration restored and its four verified routes re-verified; disposable agent/sessions removed. The streamed final-request receipt shows `output_ceiling_enforced=true` but `reserved_output_tokens=null` in this binary, so the limit was proven behaviorally. One provider only; Mac only. [Receipt](c33-rest-model-restart-macos-pass.json), [evidence](rest-model-restart/246c766c-a558-4048-a119-71f521acdcc2.json). Credits B36; does not make U20 a full legacy pass.

### C34 — Intent routing follows the live eligible roster

- **Given:** saved math/writer agents plus one disposable, distinguishable specialist.
- **When:** make the new agent eligible, route its task, then disable/remove its eligibility and route a fresh task and an unmatched intent.
- **Then:** selection uses only the current eligible roster, the chosen agent performs the task, and no-match follows the documented fallback/refusal; selection arrives before answer text.
- **Prerequisites/effects:** A; owned roster changes only. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** roster snapshots, classifier/selection events, actual agent IDs and final answers. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U06.live-roster-intent`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27, attempt 5** ([receipt](c34-live-roster-routing-macos-pass.json)): `scripts/behaviors/live-roster-routing.cucumber.mjs`, 1 scenario / 13 steps passed in 1m 4.9s on binary `b4e68525…` (liter/MiniMax-M3). The new specialist was selected by intent only while eligible and answered with its private signature, with no reasoning streamed. After deletion, a fresh plant task went to `general-purpose` and the unmatched cheese request to `default-agent`, both on the current roster. Raw evidence was reviewed. The log confirms the routed child ran with the 8,192 delegated output allowance. Attempts 1–4 failed and their receipts are kept: test script; reasoning counted as answer; router refusal; reasoning-only child. Product fixes: OpenSpec `fix-orchestrator-routing-reasoning` and `raise-delegated-child-output-allowance`. One pass only; it does not prove the old 1,000 cap would have failed on this run.

### C35 — Browser conversation reloads from saved history

- **Given:** the shipped browser UI and a disposable custom agent with a fresh fact-bearing conversation.
- **When:** reload the page, reopen that conversation and send a follow-up containing no earlier fact.
- **Then:** visible saved turns return and the follow-up uses the earlier fact; no client-invented transcript or duplicate run.
- **Prerequisites/effects:** B. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** browser trace/screenshots, session IDs, relevant HTTP/events and answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U24.shipped-ui.custom U02.session-continuation`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c35-browser-history-reload-macos-pass.json)): Thread survived a quick reload with its title, reopened with the READY turn, follow-up answered {CASE-9ae36d8c…, 806} from history; 2 POSTs, 2 runs, no page errors. Required four browser fixes and one new GET route (see receipt).

### C36 — Team follow-up reassembles child context and dependencies

- **Given:** the saved specialists, an owned team and a first-round private fact/artifact.
- **When:** submit a dependent join before its parents finish, then finish the parents and submit the legitimate next round using their actual artifacts.
- **Then:** premature execution is refused with no model dispatch; the later round remembers the first-round fact through the child thread and consumes the exact parent artifacts once.
- **Prerequisites/effects:** T. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** task dependency states, child thread/run IDs, source artifact IDs, dispatch ledger and final article. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U17.team-dependencies U18.team-multicall-context`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c36-team-follow-up-macos-pass.json)): `scripts/behaviors/team-follow-up.cucumber.mjs`, 9/9 steps in 1m 36s (attempt 1). Premature claim/admit refused (409, no attempt); private fact reached the join only via the note artifact; join consumed exactly the two parent artifacts once. Raw evidence reviewed.

### C37 — Approved native tool results reach the answer; denial has no effect

- **Given:** an owned native tool with an inspectable private result/effect and a model that can call it.
- **When:** approve one request and deny a second request to that tool.
- **Then:** the approved effect happens once and its real result informs the answer; the denied request has no tool effect and is reported honestly.
- **Prerequisites/effects:** N. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** tool-call/result pairing, approval decisions, before/after effect record, genuine final answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U22.native-tool-approval`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c37-native-tool-approval-macos-pass.json)): `scripts/behaviors/native-tool-approval.cucumber.mjs`, 7/7 steps in 41.6s (attempt 3; attempts 1–2 failed on fixture policy and a script parse bug). Approved file_write wrote the 55-byte owned note once and the answer reported the tool's bytes_written; the denied call left no file and was reported as not written. Raw evidence reviewed.

### C38 — Real MCP tool discovery, approval and result use

- **Given:** an externally provisioned owned MCP peer exposing a tiny record tool.
- **When:** discover the peer, approve a write/read request, then deny a further write.
- **Then:** the actual MCP peer is called, its value reaches the answer, and denial leaves the record unchanged.
- **Prerequisites/effects:** M. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** peer discovery and call log, paired results, record snapshots, final answer; no simulated positive tool. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U22.mcp-tool-approval`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c38-mcp-tool-approval-macos-pass.json)): `scripts/behaviors/mcp-tool-approval.cucumber.mjs`, 9/9 steps in 48.0s (attempt 5; attempts 1–4 were fixture/script issues). Native run with a per-run loopback MCP grant (ambient sharing is disabled here); approved record_write/record_read hit the real peer once each and the answer reported its value and revision; the denied write left the peer untouched. Raw evidence reviewed.

### C39 — Durable memory can be recalled, updated and removed

- **Given:** an enabled owned memory store and a document-independent private fact.
- **When:** save it, recall from a fresh conversation, update it, restart, recall the new value, then delete it and ask in another fresh conversation.
- **Then:** only the current saved value is recalled with actual memory provenance; deleted content is unavailable and stays absent after reload.
- **Prerequisites/effects:** D, R. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** memory records/history, cold-load identities, recall events and fresh answers; no KB or transcript supplies the fact. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.memory-lifecycle`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c39-memory-lifecycle-macos-pass.json)): `scripts/behaviors/memory-lifecycle.cucumber.mjs`, 10/10 steps in 1m 33.8s (attempt 2; attempt 1's oracle only accepted prompt injection). Recall came from native memory tools: saved code recalled; updated code recalled alone after restart with history; deleted code absent and answered UNKNOWN, also after another restart. Observation: no prompt-injected memory context was emitted. Raw evidence reviewed.

### C40 — Standard skill discovery and reload change usable behavior

- **Given:** a tiny owned standard skill pack with top-level/nested entries and a non-skill file.
- **When:** discover/reload it, modify one skill body and reload again.
- **Then:** real manifest skills appear once, non-skills are excluded, and a new prompt uses the changed body rather than stale cached content.
- **Prerequisites/effects:** F. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** manifest paths/hashes, reload response, catalog identity, activation/body hash and answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U09.skill-discovery-scope`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c40-skill-reload-macos-pass.json)): `scripts/behaviors/skill-reload.cucumber.mjs`, 7/7 steps in 19.0s (attempt 3, binary 52360269 after two operator-approved fixes in SkillService::refresh: persist newly discovered skills; live filesystem result supersedes its persisted copy). Discovery exact; original rule 156; changed rule 248 after reload; removed pack gone. Raw evidence reviewed.

### C41 — Explicit skill attachment works in catalog mode

- **Given:** catalog mode with an eligible skill containing a private calculation rule.
- **When:** attach that skill to a fresh request instead of relying on implicit overlay.
- **Then:** the body is activated as an attachment and the answer follows its private rule; suggestion alone is not credited as activation.
- **Prerequisites/effects:** F. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** activation source attachment, eligible IDs, actual private-rule answer and run manifest. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c41-skill-attachment-macos-pass.json)): `scripts/behaviors/skill-attachment.cucumber.mjs`, 5/5 steps (attempt 1). skill_attachments activated the skill (selection_method attachment; matcher did not accept it) and the answer 225 followed its private fee rule. Raw evidence reviewed.

### C42 — Skill scope and lower-scope policy cannot widen access

- **Given:** owned skills A/B/C, a parent selection of A/B with B denied, and two externally provided conversation contexts; one context disables A.
- **When:** request lower-scope selected[A,C], then none, then an allowed selected[A] in the enabled context; restart and read the saved disabled scope.
- **Then:** selected[A,C] resolves to selected[A] before the scoped disable and cannot admit C/B; none has empty eligible/active sets; the disabled context never loads A, while the allowed context uses A's private rule and the disabled scope survives restart.
- **Prerequisites/effects:** F, R; actual resource policy vocabulary is src/uar/domain/policy.rs:82–109 and resolution at 815–900; no test credentials. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** fixed parent/request/effective ID sets and modes, scope state, typed failures or empty selection, no forbidden body/tool/fact and genuine allowed answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U09.skill-discovery-scope U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c42-skill-scope-policy-macos-pass.json)): `scripts/behaviors/skill-scope-policy.cucumber.mjs`, 10/10 steps (attempt 3; attempts 1–2 were test-design/script issues). selected[A,C]→[A] with B/C ineligible (196); none admits nothing (UNKNOWN); conversation-scoped disable blocks A there (disabled, UNKNOWN) while another conversation uses A (252); disable survives restart. Raw evidence reviewed.

### C43 — Active skill survives compaction within its reattachment budget

- **Given:** an activated skill whose private rule will be needed later and a small reattachment allowance.
- **When:** force genuine conversation compaction and send a later rule-dependent task.
- **Then:** the latest permitted skill body is reattached within the allowance and the real answer still follows it; excess bodies do not silently widen the budget.
- **Prerequisites/effects:** F, C. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** pre/post activation/body hashes, actual compaction and reattachment counts, final answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U11.bounded-skill-reattachment`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c43-skill-reattachment-macos-pass.json)): `scripts/behaviors/skill-reattachment.cucumber.mjs`, 6/6 steps (attempt 4, binary cc1dbb81 after the operator-approved summarizer fix that excludes inline reasoning). 7 applied summaries (13505→12581 tokens); small skill whole (34), large truncated (55), total 149 ≤ 150; answer 66. Raw evidence reviewed.

### C44 — Truncate-middle actually removes the middle

- **Given:** a history with distinct first/middle/latest sentinels and explicit keep_first/keep_last limits.
- **When:** cross the structural threshold and ask about facts in retained groups.
- **Then:** the structural receipt and provider-input fragment evidence show the middle removed, the required first/latest groups retained and their facts usable.
- **Prerequisites/effects:** C. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** source/prepared hashes and groups, changed=true with the actual effect, bounded final request and answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U12.context-strategy-effects`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c44-truncate-middle-macos-pass.json)): `scripts/behaviors/context-structural.cucumber.mjs` truncate-middle scenario, 5/5 steps (attempt 1). Receipt 12→5 changed; answer used first and latest codes, middle UNKNOWN. Raw evidence reviewed.

### C45 — Sliding-window retains the intended recent groups

- **Given:** a history with distinguishable old/recent groups and an explicit small window.
- **When:** cross that window and ask a follow-up about a recent retained fact.
- **Then:** old eligible groups leave prepared input, the most recent complete groups remain in order and the answer uses the retained fact.
- **Prerequisites/effects:** C. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual retained/dropped group IDs and counts, serialized request budget and answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U12.context-strategy-effects`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c45-sliding-window-macos-pass.json)): `scripts/behaviors/context-structural.cucumber.mjs` sliding-window scenario, 5/5 steps (attempt 2; attempt 1 failed on answer-key naming). Receipt 12→5 changed; recent code used, oldest UNKNOWN. Raw evidence reviewed.

### C46 — Whole-request overflow refuses before an over-budget dispatch

- **Given:** a small real request allowance with instructions, tool schema, retrieval and output reservation contributing to it.
- **When:** submit a request whose non-removable content exceeds the allowance.
- **Then:** a typed budget refusal occurs before provider dispatch; required content is not silently dropped and each included component/reservation is accounted for.
- **Prerequisites/effects:** C, N; tiny owned KB. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** component count quality, final-wire/input allowance, reservation and zero dispatched over-budget call; no invented exact tokenizer guarantee. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U13.whole-request-budget`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c46-request-overflow-macos-pass.json)): `scripts/behaviors/request-overflow.cucumber.mjs`, 6/6 steps (attempt 1). Small-window route (4096/512): typed context_overflow 'requires 12424 … allowance 312 … excludes 3272 tool/world tokens and the output reserve'; zero admitted model calls; no answer. Raw evidence reviewed.

### C47 — Replacing a KB source removes the old fact

- **Given:** an owned indexed document with one private fact and a replacement source containing a different value.
- **When:** delete the old document, upload/index the replacement and ask in a fresh conversation.
- **Then:** the answer uses only the replacement fact with its current citation; old document/fact is absent from retrieval.
- **Prerequisites/effects:** K. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** old/new document IDs and hashes, indexing completion, retrieval hits and grounded answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U14.knowledge-lifecycle`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c47-kb-source-replace-macos-pass.json)): `scripts/behaviors/kb-source-replace.cucumber.mjs`, 6/6 steps in 9.9s (attempt 1). Old note deleted; replacement indexed; retrieval returned only it; fresh answer {724, NEW seal, [1]} cited the replacement; no trace of the old fact. Raw evidence reviewed.

### C48 — Same-dimension embedding change requires regeneration

- **Given:** two genuine embedding models with the same declared dimension but different fingerprints and one retained indexed source.
- **When:** switch the active binding, attempt retrieval, then regenerate from the retained source.
- **Then:** old vectors are rejected despite equal dimension; completed new-generation vectors retrieve the same private fact without reupload.
- **Prerequisites/effects:** K, E. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** old/target fingerprints, actual mismatch, regeneration state, retained source identity and retrieval. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U15.embedding-regeneration`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c48-embedding-same-dimension-macos-pass.json)): `scripts/behaviors/embedding-same-dimension.cucumber.mjs`, 6/6 steps (attempt 3, using the host embedding fixture; attempts 1–2 blocked by the owned config's blackhole endpoint). Same-width switch refused (embedding_space_mismatch); regeneration completed; fact retrieved (0.834). Host config restored. Raw evidence reviewed.

### C49 — Interrupted KB regeneration resumes honestly

- **Given:** a disposable multi-chunk corpus and an owned regeneration job with observable progress.
- **When:** interrupt the owned UAR after regeneration has started but before completion, restart and resume using the designed operation.
- **Then:** the job has an honest recoverable state, retained source survives, completed retrieval uses one target generation and no false ready state or mixed vectors.
- **Prerequisites/effects:** K, E, R. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** job states before/after actual interruption, generation/chunk provenance and recovered retrieval. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U15.embedding-regeneration`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c49-regeneration-resume-macos-pass.json)): `scripts/behaviors/regeneration-resume.cucumber.mjs`, 7/7 steps in 2m 57s (attempt 5, binary b80d673b after two operator-approved ingestion conflict-retry fixes). Interrupted at 1/60; after restart running 23/60, no false complete, old space kept; resumed to 60/60; target space only at completion; note-17 fact retrieved. Raw evidence reviewed.

### C50 — A user can cancel an active run

- **Given:** a genuine long-enough run and an owned cancellation endpoint.
- **When:** cancel after live execution starts and inspect its terminal state.
- **Then:** the run is cancelled, no later success is reported, outstanding usage/effects are settled or explicitly uncertain, and unrelated subsequent prompting remains usable.
- **Prerequisites/effects:** A; optional pending native tool. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** live run/cancel/terminal events, dispatch/effect accounting and later health of ordinary prompt. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U22.native-tool-approval U23.failure-honesty`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c50-run-cancellation-macos-pass.json)): `scripts/behaviors/run-cancellation.cucumber.mjs`, 7/7 steps in 9.7s (attempt 2; attempt 1's oracle missed the run's explicit uncertainty diagnostic). Live run cancelled after streaming; terminal cancelled, no later events; usage pending with accounting_unresolved reported; fresh prompt answered 42. Raw evidence reviewed.

### C51 — Provider failure produces an honest recoverable result

- **Given:** a named owned provider fault path with a request admitted for a genuine configured model.
- **When:** disconnect or return a controlled failure during that request, then remove the fault and send a fresh request.
- **Then:** the failed run never reports success or fabricates an answer, accounting is settled/explicitly unknown, and the fresh run succeeds; no silent model switch.
- **Prerequisites/effects:** X. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** fault admission, raw failure/terminal, call disposition and restored genuine answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U23.failure-honesty`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c51-provider-failure-macos-pass.json)): `scripts/behaviors/provider-failure.cucumber.mjs`, 7/7 steps in 26.2s (attempt 2; attempt 1 failed only in provider-restore cleanup). Owned fault proxy cut the Liter stream mid-response: run error provider_transport_failed, no answer, usage pending (accounting_unresolved), same model. Fault cleared: fresh run answered 42 on liter/MiniMax-M3. Provider restored and five models re-verified. Raw evidence reviewed.

### C52 — Initial and repeat startup preserve persisted settings

- **Given:** an owned initially empty store, explicit harmless seed settings and an external host fixture operation restricted to aging the timestamp of one later-created disposable terminal session.
- **When:** perform initial startup, save an operator override, perform repeat startup, create one terminal/fresh-control session pair, have the host fixture age only the terminal session past retention, then complete a new turn that publishes production session activity.
- **Then:** built-ins are seeded once, the override remains effective, and the actual run-retention worker removes only the expired session while preserving the fresh control and protected definitions; a second activity signal is idempotent.
- **Prerequisites/effects:** D, R; host prepares only the captured owned session timestamp after repeat startup and before the triggering turn, never advances the system/DB clock; if the source-backed host fixture is unavailable this case is BLOCKED. Production seed path src/server.rs:1417 and run-retention owner src/uar/runtime/manager.rs:1390–1452; observe through session APIs/persisted owned records, not a fictional maintenance endpoint. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** empty-store/startup identities, built-in counts and saved revisions, expired/fresh record timestamps, activity and authoritative removal/preservation observations. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U21.startup-maintenance`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c52-startup-retention-macos-pass.json)): Disposable UAR on a fresh Surreal 3.3.0 store: 5 seed lines then 0; override {4321, 12345} kept with its revision; host-aged terminal session purged on the next turn (one purge line), control and trigger kept; second signal purged nothing; full teardown, owned UAR untouched.

### C53 — Browser shows orchestrator agent selection before the answer

- **Given:** the shipped UI, saved math/writer specialists and an orchestrator conversation.
- **When:** send a writing task, then a math task in that conversation through the browser.
- **Then:** the visible selection follows each intent before its specialist answer, both actual specialist IDs match execution and no duplicate answer/run is created.
- **Prerequisites/effects:** B, A. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** browser trace of selection state and output, server selection and run IDs. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U24.shipped-ui.orchestrator U07.official-agui-selection-replay`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c53-browser-agent-selection-macos-pass.json)): Writing then math in one browser conversation: each uar.agent.selected (intent) precedes text on the wire and is visible before the answer (10.9 s, 4.4 s); one child run per turn matching the selection; 2 POSTs, 2 root runs; math 22213. Required the duplicate-message and save-every-bubble UI fixes plus revised script checks (see receipt).

### C54 — A team recovers after cancellation/restart without duplicate work

- **Given:** an owned team with one settled task and one active task.
- **When:** cancel the active task, restart and resume only work that the designed recovery state permits.
- **Then:** completed artifacts remain unchanged, cancelled/unknown work is honest, no settled external effect/model call is silently repeated and a legitimate dependent conclusion can be produced.
- **Prerequisites/effects:** T, R. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** durable task/thread states, artifact/usage identities before and after, permitted recovery and conclusion. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U19.team-cancel-recovery`. Historical IDs remain a planning map, not retroactive full-contract passes.
- **Result — PASS on Mac, 2026-10-27** ([receipt](c54-team-recovery-macos-pass.json)): `scripts/behaviors/team-recovery.cucumber.mjs`, 14/14 steps (attempt 3; attempts 1–2 failed only on the writer calling denied team tools). Settled tally byte-identical across restart/recovery; cancelled note honest and not repeated; recover re-dispatched nothing; replacement join concluded (486, SEEDLOCK). Raw evidence reviewed.

## Additional supported choices and resilience

All E01–E28 are **PLANNED / NOT RUN**. These retain supported options without blocking completion of common runnable work. The explicit engine/method comes from production enums, not a test-only alias. E16–E24 are the nine not-yet-credited matching implementations; keyword and local_embedding already have C16/C17 evidence. No fallback to another method earns a pass.

### E01 — Hierarchical context produces real summary tiers

- **Given:** a history long enough for short/mid/long tiers with early and recent private facts.
- **When:** cross hierarchical thresholds and ask a later fact-dependent task.
- **Then:** actual mid-term summary and long-term facts are generated from the captured source; required facts remain useful within their tier budgets.
- **Prerequisites/effects:** C. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** source/tier hashes, real summary calls, tier token counts and answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U12.context-strategy-effects`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E02 — Auto context resolves for the selected model

- **Given:** a configured auto strategy and a known model context window.
- **When:** cross its effective threshold and ask about a designed retained fact.
- **Then:** the actual resolved strategy matches the documented model rule and its observed effect preserves that fact within budget.
- **Prerequisites/effects:** C. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** resolved model/window/strategy, pre/post groups and final answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U12.context-strategy-effects`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E03 — Memory opt-out and retention survive restart

- **Given:** an owned 30-day capture policy with one externally provisioned automatic memory first_capture eight days in the past, a fresh automatic control memory and their actual content/control IDs.
- **When:** PUT the real 7-day policy so the old record's expiry becomes first_capture+7d, observe production purge/visibility, opt out, send a new private fact, then restart.
- **Then:** the aged fact becomes unqueryable and is purged by the real control cleanup, the fresh control stays valid, the opt-out survives restart and the new fact is not late-captured.
- **Prerequisites/effects:** D, R; host provisions only the owned aged memory/control consistently before the test. Allowed days are 7/30/90/365; no short fake TTL, clock override or waiting seven days. Actual shortening/visibility/purge: src/uar/memory/controls_store.rs:57–109,248–292; restart resumes purge at initialize. If the aged fixture is absent, BLOCKED. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** policy readback, first_capture/effective_expires_at, real control status/purge, fresh-memory preservation, cold policy and absence of opted-out fact. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.memory-consent`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E04 — Routing honors required capabilities and availability

- **Given:** two already provisioned genuine routes with different declared capabilities and one owned cooldown condition.
- **When:** request a tool/vision/format-qualified route, exclude its preferred route, and request an impossible capability set.
- **Then:** only eligible healthy routes are used for actual work; no-match is explicit and never silently relaxes requirements.
- **Prerequisites/effects:** A, X; two bindings. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** requirements/candidates, actual selected dispatch and capability result; no-match zero dispatch. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.routing-capabilities`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E05 — Structured output follows the supplied schema

- **Given:** a genuine model that advertises structured output and a schema containing a fresh discriminator absent from the prompt.
- **When:** request a response through the public response_format field.
- **Then:** the returned final payload validates against that exact schema and contains its discriminator; plain prompt compliance alone does not pass.
- **Prerequisites/effects:** A; schema-capable binding. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** request response_format, actual forwarded format/receipt and final parsed result. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.formats-vision`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E06 — Vision answers from actual image content

- **Given:** a genuine vision binding and a locally generated image containing unpredictable simple symbols.
- **When:** send the image without describing its hidden symbols in text.
- **Then:** the final answer identifies the actual image content and the real dispatch carries the image on a vision-capable route.
- **Prerequisites/effects:** A; owned image. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** image hash, payload/attachment reference, actual route and independently known answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.formats-vision`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E07 — Compaction preserves required tool and image groups

- **Given:** a conversation containing a complete tool-call/result pair and an image needed for a later answer.
- **When:** force reduction and restart, then ask a question requiring that retained group.
- **Then:** required groups remain intact and correctly paired with provenance; image/tool facts remain usable or admission explicitly refuses instead of silently corrupting context.
- **Prerequisites/effects:** C, N, R; vision binding. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** group identities and hashes, no orphan tool result, cold continuation and final answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.protected-multimodal-context`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E08 — Unknown tool effect is not silently repeated

- **Given:** an owned MCP tool that commits a record then loses its response.
- **When:** request the effect, observe the uncertain result and reconnect/resume the same operation.
- **Then:** the committed effect remains exactly once, the runtime preserves uncertainty and no automatic retry claims success or duplicates it.
- **Prerequisites/effects:** M, X. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** peer effect log, response-loss point, unknown disposition, reconnect and unchanged effect count. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U23.unknown-tool-effect`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E09 — External owner contexts isolate runs and conversation history

- **Given:** two identity endpoints provisioned outside the test and a private fact under owner A in one fresh session S.
- **When:** use owner B's endpoint to GET A's run and events and POST its cancel operation, then prompt with the same public session ID S as B; restart and follow up as A.
- **Then:** foreign run read/events/cancel return 403/404 without changing A's run; B's legitimate inference is in an isolated session partition and cannot return A's fact, while A still recalls it after restart.
- **Prerequisites/effects:** two external base URLs, A, R; exact run endpoints /api/uar/runs/{id}, /events, /cancel and /v1/chat/completions; no auth headers or credentials. This does not assert global agent definitions are owner-private. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** operation/result table, B's isolated answer and actual run identity, unchanged A run, two distinct owned session records and A's cold recall. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U23.owner-isolation`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E10 — Reconnecting selection streams does not dispatch again

- **Given:** one orchestrator run with captured selection event IDs and a continuation request identity.
- **When:** reconnect using its public cursor and replay the same admitted request, then make a distinct next turn.
- **Then:** replay preserves event/run identity and yields no second dispatch; the next turn performs a fresh legitimate selection.
- **Prerequisites/effects:** A. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** original/replayed ordered events, cursor, run/request IDs and unchanged dispatch count. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U07.official-agui-selection-replay`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E11 — Updated and deleted agent revisions survive restart

- **Given:** one disposable agent with a changed instruction revision.
- **When:** restart, prompt that saved revision, delete it, restart and try to select its old ID.
- **Then:** the changed instructions remain effective before deletion and the deleted definition cannot be read/selected after cold startup.
- **Prerequisites/effects:** A, R. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** revision snapshots, marker-bearing answer, DELETE and cold absence/no dispatch. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U03.custom-agent`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E12 — KB deletion remains deleted after cold startup

- **Given:** an owned indexed KB linked to a disposable agent.
- **When:** delete the KB, restart and inspect direct retrieval and the saved agent in a fresh conversation.
- **Then:** source/index are absent after restart, the remaining agent is usable and cannot cite or retrieve that deleted private knowledge.
- **Prerequisites/effects:** K, R. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** KB/document/index absence, unchanged agent link disposition, fresh non-grounded answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U14.knowledge-lifecycle`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E13 — The model explicitly activates a catalog skill

- **Given:** catalog mode and one eligible skill whose body alone contains the required private rule.
- **When:** prompt a task that requires activate_skill rather than attaching the body in advance.
- **Then:** a genuine model activation occurs and its real rule/tool result reaches the answer, with no implicit body loading.
- **Prerequisites/effects:** F. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** catalog-only pre-call manifest, activate_skill call/outcome, next-call body and final answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E14 — Skill activation count limit is enforced

- **Given:** an agent with max_active set to one, two eligible private skills and one already activated.
- **When:** attempt to activate the second skill.
- **Then:** a typed max_active refusal occurs, the first skill remains usable and the second body/tools/fact are never loaded.
- **Prerequisites/effects:** F. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** count limit, activation outcomes, body/tool manifests and no forbidden effect. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U22.skill-activation-count-limit`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E15 — One prompt discriminates lexical from semantic selection

- **Given:** a controlled skill pair where a lexical decoy and semantic target have different private outcomes.
- **When:** send the same prompt once under each selected matcher.
- **Then:** recorded scores/methods choose the designed different winners and each answer consumes only its winner's private rule.
- **Prerequisites/effects:** F, E; reuse prior fixtures as data, not old tests. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** identical prompt hash, zero/known lexical controls, scored winners and actual answers. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U25.lexical-semantic-disagreement`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E16 — Service embedding matcher selects and uses a skill

- **Given:** a target/decoy skill pair and the explicit service embedding binding.
- **When:** send a discriminating prompt through normal agent execution.
- **Then:** requested_method and effective_method both name skill_service.embedding; the accepted target's private rule is used and the decoy is absent.
- **Prerequisites/effects:** F, E. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** scored candidates, actual matching method/model, activation and final private-rule answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E17 — Service hybrid matcher selects and uses a skill

- **Given:** a target/decoy skill pair and the explicit service hybrid binding.
- **When:** send a discriminating prompt through normal agent execution.
- **Then:** requested_method and effective_method both name skill_service.hybrid; the accepted target's private rule is used and the decoy is absent.
- **Prerequisites/effects:** F, E. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** scored candidates, actual matching method/model, activation and final private-rule answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E18 — Service llm matcher selects and uses a skill

- **Given:** a target/decoy skill pair and the explicit service llm binding.
- **When:** send a discriminating prompt through normal agent execution.
- **Then:** requested_method and effective_method both name skill_service.llm; the accepted target's private rule is used and the decoy is absent.
- **Prerequisites/effects:** F; genuine classifier model. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** scored candidates, actual matching method/model, activation and final private-rule answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E19 — Intent-classifier rules selects and uses a skill

- **Given:** a target/decoy skill pair and explicit intent_classifier rules backend.
- **When:** send its discriminating prompt through a real run.
- **Then:** the actual backend remains legacy_classifier.rules with no substitute; the selected body's private rule affects the answer.
- **Prerequisites/effects:** F; genuine model when llm. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual backend, candidates/activation and private-rule answer; missing component/model is BLOCKED. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E20 — Intent-classifier tfidf selects and uses a skill

- **Given:** a target/decoy skill pair and explicit intent_classifier tfidf backend.
- **When:** send its discriminating prompt through a real run.
- **Then:** the actual backend remains legacy_classifier.tfidf with no substitute; the selected body's private rule affects the answer.
- **Prerequisites/effects:** F; genuine model when llm. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual backend, candidates/activation and private-rule answer; missing component/model is BLOCKED. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E21 — Intent-classifier hybrid selects and uses a skill

- **Given:** a target/decoy skill pair and explicit intent_classifier hybrid backend.
- **When:** send its discriminating prompt through a real run.
- **Then:** the actual backend remains legacy_classifier.hybrid with no substitute; the selected body's private rule affects the answer.
- **Prerequisites/effects:** F, E. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual backend, candidates/activation and private-rule answer; missing component/model is BLOCKED. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E22 — Intent-classifier wasm selects and uses a skill

- **Given:** a target/decoy skill pair and explicit intent_classifier wasm backend.
- **When:** send its discriminating prompt through a real run.
- **Then:** the actual backend remains legacy_classifier.wasm with no substitute; the selected body's private rule affects the answer.
- **Prerequisites/effects:** F; actual owned WASM component. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual backend, candidates/activation and private-rule answer; missing component/model is BLOCKED. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E23 — Intent-classifier localembedding selects and uses a skill

- **Given:** a target/decoy skill pair and explicit intent_classifier localembedding backend.
- **When:** send its discriminating prompt through a real run.
- **Then:** the actual backend remains legacy_classifier.localembedding with no substitute; the selected body's private rule affects the answer.
- **Prerequisites/effects:** F, E. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual backend, candidates/activation and private-rule answer; missing component/model is BLOCKED. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E24 — Intent-classifier llm selects and uses a skill

- **Given:** a target/decoy skill pair and explicit intent_classifier llm backend.
- **When:** send its discriminating prompt through a real run.
- **Then:** the actual backend remains legacy_classifier.llm with no substitute; the selected body's private rule affects the answer.
- **Prerequisites/effects:** F; genuine model when llm. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** actual backend, candidates/activation and private-rule answer; missing component/model is BLOCKED. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E25 — Inherited skill selection preserves the parent set

- **Given:** skill universe {A,B,C}, parent selected[A] with C denied, a turn policy inherit and catalog presentation.
- **When:** prompt using the inherited allowed skill A with no lower-scope IDs.
- **Then:** the effective mode is selected and exact eligible set is {A}; B/C bodies and tools stay absent and the genuine answer uses A's private rule.
- **Prerequisites/effects:** F; fixed ID labels resolve to disposable actual skill IDs; src/uar/domain/policy.rs:815–900. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** parent/turn/effective policies, exact catalog/active IDs, body hashes and final private-rule answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E26 — All policy exposes every eligible catalog skill

- **Given:** an owned skill universe of exactly {A,B}, no inherited narrowing/denials and catalog presentation.
- **When:** set turn skills.mode=all and ask a task that uses A.
- **Then:** the effective mode remains all, eligible/catalog IDs equal {A,B} and the answer uses actually activated A; B being catalog-eligible does not itself mean its body was loaded.
- **Prerequisites/effects:** F; isolate the owned catalog/universe, max_active at least one; src/uar/domain/policy.rs:815–900. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** exact universe and effective mode/IDs, actual catalog/body distinction and genuine rule-dependent answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E27 — Auto policy chooses within a narrowed eligible set

- **Given:** universe {A,B,C}, parent selected[A,B], a lower auto request and a discriminating prompt matching A while C is the tempting out-of-parent decoy.
- **When:** execute under catalog presentation and permit genuine activation only after the effective eligibility set is formed.
- **Then:** the lower auto request cannot widen the parent: effective mode is selected with exact IDs {A,B}, C is absent, and actual chosen/activated A drives the answer rather than the excluded C.
- **Prerequisites/effects:** F; source src/uar/domain/policy.rs:815–900; this tests lower-scope auto/narrowing, not a repeat of the already verified implicit keyword scenario. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** requested auto, parent/effective set and mode, scored/activated IDs, excluded body/tool absence and private-rule answer. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.

### E28 — Below-threshold or ambiguous skills stay inactive

- **Given:** an owned target/decoy catalog under legacy_overlay with a configured threshold/margin, no attachment and no explicit/model activation path.
- **When:** submit a below-threshold prompt and then a controlled unresolved tie under the same implicit-only policy.
- **Then:** neither request implicitly loads a body or private fact; scores and refusal/no-selection match the configured threshold/margin.
- **Prerequisites/effects:** F. Clean up only owned disposable resources and restore changed settings.
- **Evidence / failure:** candidate scores, threshold/margin, no activated body/tool or leaked private marker. Any missing required outcome is FAIL; unavailable prerequisites are BLOCKED with their exact reason.
- **Trace:** `U10.skill-matching-strategies U11.skill-activation-governance`. Historical IDs remain a planning map, not retroactive full-contract passes.

## Remaining public client and wire contracts

All I01–I20 are **PLANNED / NOT RUN**. Each row has this concrete contract unless its stated special oracle overrides it:

- **Given:** the already running real UAR, one configured model, and one disposable saved agent with a private instruction marker (implicit-default cases deliberately omit agent_id).
- **When:** use exactly the public surface/mode named below to answer a fresh input-dependent task and, where the public protocol supports it, send one fact-dependent follow-up using its designed session/context or client-transcript contract.
- **Then:** the complete answer is correct, saved instructions/default selection are effective, public correlation and successful terminal framing are valid, and the follow-up uses the original fact. Streaming must deliver genuine content increments before terminal; reasoning is not an answer. Client-supplied replay is allowed only where that client protocol requires it and must not be mislabeled server-memory continuity.
- **Evidence / failure:** retain requests, raw responses/events, client/package revision, actual agent/model/run/session and termination; wrong answer, ignored instructions, wrong model, malformed framing or missing continuation is FAIL. An unavailable optional client/feature is BLOCKED, not waived. Ownership is the scenario author plus the corresponding interface maintainer.
- **Effects:** two small genuine turns where supported, temporary agent/session and read-only run inspection; delete only owned resources and leave UAR running. SDK steps are Node orchestrated and invoke the actual shipped language client, not an HTTP imitation; they require the externally prepared language/toolchain/artifact.

| ID | Specific public contract | Trace / special prerequisites and oracle |
|---|---|---|
| I01 | Chat legacy AG-UI framing: /api/chat/completion, stream_mode=agui | `U05.chat.agui.default U05.chat.agui.custom U05.openai.agui.default U05.openai.agui.custom`. |
| I02 | Chat dual framing: /v1/chat/completions, stream_mode=dual | `U05.chat.dual.default U05.chat.dual.custom U05.openai.dual.default U05.openai.dual.custom`. Both vocabularies correlate to one actual run/dispatch; no doubled answer. |
| I03 | Native run admission with legacy AG-UI stream: POST /api/uar/runs then GET its stream, default format | `U05.native.agui.default U05.native.agui.custom`. |
| I04 | Native run admission with official AG-UI stream: POST /api/uar/runs then GET its stream?stream_mode=agui_spec | `U05.native.agui_spec.default U05.native.agui_spec.custom`. |
| I05 | Anthropic streaming completion: /v1/messages with stream=true and implicit default selection | `U05.anthropic.stream.default U05.anthropic.stream.custom U05.anthropic.stream.implicit`. Omitted agent_id must resolve the protected default agent, never silently drop the prompt. |
| I06 | Legacy A2A JSON completion: legacy mounted A2A HTTP JSON-RPC send method | `U05.a2a_legacy.json.default U05.a2a_legacy.json.custom`. |
| I07 | A2A v1 streamed task: mounted v1 A2A HTTP streaming/task method | `U05.a2a_v1.stream.default U05.a2a_v1.stream.custom`. |
| I08 | ACP non-streamed request: mounted ACP JSON-RPC non-streaming operation | `U05.acp.json.default U05.acp.json.custom`. |
| I09 | python sdk unary completion: the shipped python_sdk public json client operation | `U05.python_sdk.json.default U05.python_sdk.json.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I10 | python sdk streamed completion: the shipped python_sdk public stream client operation | `U05.python_sdk.stream.default U05.python_sdk.stream.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I11 | typescript sdk unary completion: the shipped typescript_sdk public json client operation | `U05.typescript_sdk.json.default U05.typescript_sdk.json.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I12 | typescript sdk streamed completion: the shipped typescript_sdk public stream client operation | `U05.typescript_sdk.stream.default U05.typescript_sdk.stream.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I13 | rust sdk unary completion: the shipped rust_sdk public json client operation | `U05.rust_sdk.json.default U05.rust_sdk.json.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I14 | rust sdk streamed completion: the shipped rust_sdk public stream client operation | `U05.rust_sdk.stream.default U05.rust_sdk.stream.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I15 | grpc legacy unary completion: the shipped grpc_legacy public json client operation | `U05.grpc_legacy.json.default U05.grpc_legacy.json.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I16 | grpc legacy streamed completion: the shipped grpc_legacy public stream client operation | `U05.grpc_legacy.stream.default U05.grpc_legacy.stream.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I17 | grpc v1 unary completion: the shipped grpc_v1 public json client operation | `U05.grpc_v1.json.default U05.grpc_v1.json.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I18 | grpc v1 streamed completion: the shipped grpc_v1 public stream client operation | `U05.grpc_v1.stream.default U05.grpc_v1.stream.custom`. Actual client toolchain/protobuf bindings must exist; no fake success from build-only evidence. |
| I19 | Dedicated AG-UI projects A2UI state: the dedicated AG-UI agent route with an agent publishing an A2UI surface | `U08.installed-standard-a2ui-action`. Source-noted projection gap: require actual surface/state events from this route, then rendered action continuation. Chat agui_spec success cannot stand in for it. This has a known source gap, not a measured runtime FAIL. |
| I20 | Anthropic implicit default JSON selection: /v1/messages with agent_id omitted and stream=false | `U05.anthropic.json.default U05.anthropic.json.implicit`. Omitted agent_id must resolve the protected default agent, never silently drop the prompt. |

### Source-backed deduplication, not evidence inflation

Both `/api/chat/completion` and `/v1/chat/completions` mount the **same** `api_chat_completion` handler (`src/server.rs:2405,2777`). Its `StreamMode` and request schema are at `src/server.rs:5889–5967`. Test each missing serialization mode once; do not multiply it by alias and default/custom selection. Existing C01/C02 cover JSON/OpenAI SSE, C14/C15 cover chat agui_spec and A2UI interaction. I01/I02 add the missing legacy/dual serializers. Native POST-run admission is separate (I03/I04); reading a continuation stream alone did not cover it.

Existing C10/C11/C12/C13 already cover Anthropic JSON, A2A v1 JSON, ACP stream and dedicated AG-UI answer. I05/I07/I08 add their other public modes. I20 covers the separate omitted-agent branch. Actual SDK JSON/stream operations are separately named I09–I18 because a shipped client has its own request/envelope/stream processing. Sharing a backend does not certify a language client. The complete default/custom × alias × mode Cartesian product is consciously not required by this proposed behavioral baseline; original rows keep their old status. Request-target default selection is covered by C01/I05/I20, explicit saved selection by C03/C09 and interface-marker assertions. This is representative behavior coverage, not proof of every combination.

Production and contract anchors: [capability inventory](bdd-capabilities-source-2026-10-27.md), [interface inventory](bdd-interface-scope-2026-10-27.md), `src/uar/runtime/skills/service.rs:29–59`, `src/uar/settings/manager.rs:1946`, `src/uar/context/strategy.rs:12–62`, `src/uar/api/routes.rs`, `src/uar/api/acp/routes.rs`, `src/uar/api/a2a/grpc.rs`, `src/uar/api/a2a/grpc_v1.rs`, and `sdks/{python,typescript,rust}`. Interface-inventory suggestions to drop mode cells without a production contract are superseded by the explicit cells and alias proof above. No behavior of an external dependency is inferred from memory.

## Platform and persistence coverage — do not multiply blindly

102 is the number of behavior definitions. It is **not** the count of executions needed for every platform/store. Each execution receipt records the runtime platform, backend and artifact. Current 32 credits apply only to their historical Mac cohort; no exact whole-current-tree release claim is made.

| Target | Execution order and applicability | Current status |
|---|---|---|
| Mac + disk-backed Surreal | Finish selected common workflows, then additional supported options/client modes; keep existing accepted receipts. | 32 accepted outcomes; 70 proposed definitions remain. |
| Mac + PostgreSQL/pgvector | Apply the stateful contracts below to a separately owned PG instance. Never infer parity from Surreal. | No runtime coverage credited; parity remains required before claiming both adapters. |
| Linux | After the Mac pass/disposition, execute the applicable catalog against a real Linux UAR. Preserve defects for the current platform; do not block a Mac result on a Linux prerequisite. | Not run. |
| iOS | After Linux, use the real embedded Rust host. Apply actual embedded prompting, durable session/agent/config/knowledge/skill/team operations only when exposed by its production capability discovery; unsupported operations must refuse rather than use ephemeral substitutes. | Not run; simulator and device are distinct claims. |
| Windows | The Node scripts remain portable; host lifecycle/clients require a Windows adapter/executor. | Outside selected sequence; recorded missing executor stays visible. |
| Android | Follow-up decision after selected platforms. | Deferred by operator; no certification and no release-note publication in this task. |

Store-requiring scenario IDs are explicitly enumerated below. PostgreSQL must meet the **same** external predicates, including restart/multiple calls and provenance, with independent source/backend-bound receipts. This is parity scope, not an additional invented BDD behavior count. Stateless mode/client permutations do not each need another database cross-product. If a supposedly shared stateless path has a backend-specific admission/persistence difference, add that missing cell before claiming it.

**Stateful parity set:** C03–C09, C16–C54, E01–E03, E07–E28. (Inclusive ranges denote each listed ID: **71 definitions**.) This deliberately includes memory, scope, context, activation, budget, model-run settlement, teams and recovery rather than narrowing parity to KB CRUD. The server/client serializers I01–I20 and direct/capability/format cases use the same user-outcome contract and need at least one successful real PG admission/continuation witness per distinct execution adapter; full mode × store multiplication is not required without a backend-specific path. Until the 71 stateful cells and these adapter witnesses have receipts or explicit product-support dispositions, backend parity is unverified. This is not a full current-source parity claim.


**Source limit requiring explicit backend disposition:** `src/uar/memory/controls.rs:53–80` says durable memory controls require surreal-backend, and `controls_store.rs:1–18` uses the memory service's actual Surreal database. A PostgreSQL primary runtime that still uses this memory store must disclose that fact; it is not PostgreSQL memory-control parity. C39/E03 remain unverified for an independently PostgreSQL-backed memory claim. This is a source finding, not an executed failure or permission for a product fix.

For platform applicability, never mark a server HTTP/browser/gRPC operation applicable to a direct embedded iOS host merely because a remote server can answer it. The scenario author must cite the shipped embedded capability before running that cell; missing design support is an explicit unsupported/blocked disposition, not PASS. An exact platform execution-row total will follow the actual host capability map. The 102 definition denominator stays fixed unless a scenario itself changes.

## Legacy 97-case disposition

This table maps every old ID to the proposed goal(s) or an explicit deduplication. It does not overwrite the audited 28 partial / 69 uncredited historical results. The old three-repetition/authentication/signature fixture process is superseded; its receipts and failed attempts remain history. No case becomes a full legacy PASS merely from this map.

| Original ID | Proposed behavior coverage / disposition |
|---|---|
| `U01.configured-prompt` | C01, C33; existing useful prompt retained, effective configuration adds missing product evidence. |
| `U02.session-continuation` | C07, C20, C21; existing warm/cold/concurrent proof retained. No extra canonical observer ceremony. |
| `U03.custom-agent` | C03, C04, C09, E11; no create/query replay just to change the count. |
| `U04.protected-agents` | C08 plus existing C01/C04 and startup C52. The bounded delete-refusal pass stays accepted; old bundled same-run repetition is not this baseline. |
| `U20.effective-configuration` | C22, C33; actual saved/effective configuration and rejected metadata. |
| `U21.startup-maintenance` | C52. |
| `U06.live-roster-intent` | C04, C34. |
| `U07.official-agui-selection-replay` | C34, C53, E10. |
| `U05.chat.json.default` | C01 (same handler alias); representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.json.custom` | C01 (same handler alias); representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.openai.default` | C02 (same handler alias); representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.openai.custom` | C02 (same handler alias); representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.agui.default` | I01; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.agui.custom` | I01; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.dual.default` | I02; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.dual.custom` | I02; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.agui_spec.default` | C14/C15; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.chat.agui_spec.custom` | C14/C15; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.json.default` | C01/C03/C07; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.json.custom` | C01/C03/C07; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.openai.default` | C02; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.openai.custom` | C02; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.agui.default` | I01 (same handler alias); representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.agui.custom` | I01 (same handler alias); representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.dual.default` | I02; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.dual.custom` | I02; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.agui_spec.default` | C14/C16–C21; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.openai.agui_spec.custom` | C14/C16–C21; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.native.agui.default` | I03; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.native.agui.custom` | I03; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.native.agui_spec.default` | I04; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.native.agui_spec.custom` | I04; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.anthropic.json.default` | I20; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.anthropic.json.custom` | C10; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.anthropic.stream.default` | I05; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.anthropic.stream.custom` | I05; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.a2a_legacy.json.default` | I06; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.a2a_legacy.json.custom` | I06; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.a2a_v1.json.default` | C11; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.a2a_v1.json.custom` | C11; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.a2a_v1.stream.default` | I07; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.a2a_v1.stream.custom` | I07; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.acp.json.default` | I08; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.acp.json.custom` | I08; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.acp.stream.default` | C12; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.acp.stream.custom` | C12; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.anthropic.json.implicit` | I20; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.anthropic.stream.implicit` | I05; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.python_sdk.json.default` | I09; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.python_sdk.json.custom` | I09; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.python_sdk.stream.default` | I10; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.python_sdk.stream.custom` | I10; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.typescript_sdk.json.default` | I11; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.typescript_sdk.json.custom` | I11; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.typescript_sdk.stream.default` | I12; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.typescript_sdk.stream.custom` | I12; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.rust_sdk.json.default` | I13; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.rust_sdk.json.custom` | I13; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.rust_sdk.stream.default` | I14; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.rust_sdk.stream.custom` | I14; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_legacy.json.default` | I15; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_legacy.json.custom` | I15; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_legacy.stream.default` | I16; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_legacy.stream.custom` | I16; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_v1.json.default` | I17; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_v1.json.custom` | I17; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_v1.stream.default` | I18; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U05.grpc_v1.stream.custom` | I18; representative target/mode contract. Full default/custom two-turn/accounting cross-product is superseded as a baseline requirement, not retroactively passed. |
| `U08.installed-standard-a2ui-action` | C14, C15, I19; chat/browser passes retained, dedicated route projection separately unverified. |
| `U24.shipped-ui.default` | Existing C15 supports actual shipped chat; default target selection covered by C01/I20, not a full extra UI matrix. A dedicated default-only UI cell is deferred, not passed. |
| `U24.shipped-ui.custom` | C15, C35; action proof already accepted, saved-history continuation is the missing goal. |
| `U24.shipped-ui.orchestrator` | C53. |
| `U09.skill-discovery-scope` | C40, C42. |
| `U10.skill-matching-strategies` | C16, C17, E15–E24, E28. All eleven concrete method choices are named; no hidden matcher substitution. |
| `U11.skill-activation-governance` | C41, C42, E13, E25–E28; catalog versus implicit overlay, effective policies and denial. |
| `U11.bounded-skill-reattachment` | C43. |
| `U12.context-strategy-effects` | C19, C20/C21 none, C44/C45, E01/E02. C18 does not prove truncate_middle. |
| `U13.whole-request-budget` | C18, C46. Count quality remains explicit; no false exact-tokenizer claim. |
| `U14.knowledge-lifecycle` | C05, C23, C25–C32, C47, E12. |
| `U15.embedding-regeneration` | C24, C48, C49. |
| `U16.knowledge-adapter-contract` | Stateful parity set, especially C05/C23/C24/C47–C49/E12, independently on both adapters. |
| `U17.team-dependencies` | C06, C36. |
| `U18.team-multicall-context` | C36. |
| `U19.team-cancel-recovery` | C54. |
| `U22.native-tool-approval` | C37, C50. |
| `U22.mcp-tool-approval` | C38, E08. |
| `U23.owner-isolation` | E09 via externally provisioned identity ingress; no authentication test step. |
| `U23.failure-honesty` | C50, C51, E04; a durable-write failure injection remains a deferred additional control, not covered by provider disconnect. |
| `U23.unknown-tool-effect` | E08. |
| `U25.memory-lifecycle` | C39. |
| `U25.memory-consent` | E03. |
| `U25.formats-vision` | E05, E06. |
| `U25.routing-capabilities` | E04. |
| `U25.lexical-semantic-disagreement` | E15. |
| `U25.context-threshold-restart` | Existing C20/C21 none plus C19/C44/C45/E01/E02 observed strategies. Per-strategy below-threshold cold-restart Cartesian expansion is deferred, not claimed. |
| `U22.skill-activation-count-limit` | E14. |
| `U25.protected-multimodal-context` | E07. |

## Finite per-scenario ceilings for future adoption

These are maximums, not targets or authority to execute. The smaller of these limits, remaining previously approved resources and the operator's selected-case allowance governs. Prior usage/spend is retained. No automatic verification retries or open-ended waits are permitted.

| Class / exact IDs | Hard upper bounds for one attempt |
|---|---|
| Ordinary API/skill/format/client: C34, C37–C42, C47, C48; E04–E06, E09–E18, E19–E28; I01–I18, I20 | 8 actual model dispatches including activation/auxiliary calls; 60 client HTTP/RPC operations; 300 seconds elapsed. Two ordinary protocol turns target at most 1,024 output tokens each; rule/agent output remains capped by the smaller supported request allowance. |
| Context/reattachment: C43–C46, E01, E02, E07 | At most 6 seeded user turns plus one follow-up and 12 summary/activation dispatches (19 total model calls), 100 HTTP operations, 600 seconds. Seed history at most 64 KiB UTF-8 and 16 conversation groups, two skill bodies at most 8 KiB each. Use a low explicit structural threshold so effects occur within that history; do not grow input indefinitely. |
| Restart/browser/config/memory: C33, C35, C39, C40, C42, C52, C53; E03, E09, E11, E12, I19 | 12 model dispatches, 100 HTTP/browser operations, at most two owned restarts, 900 seconds including startup. At most two owned aged records, three fresh fact-bearing turns and one follow-up. No arbitrary delay/clock override; wait only for the named actual record/terminal with the same deadline. |
| Team/cancel/fault/recovery: C36, C50, C51, C54, E08, E10 | At most 4 team tasks, 2 rounds, 20 total actual model dispatches, 120 HTTP/RPC operations, one controlled interruption and one explicitly named recovery, at most two restarts, 900 seconds. No automatic repeat of an unknown/settled effect. Each writing response ceiling is 64,000 tokens; choose a smaller task/request allowance before execution where sufficient. |
| Embedding regeneration/retrieval: C48, C49 | At most 2 tiny documents, 16 chunks and 64 KiB total source, 20 embedding requests, one binding transition and one regeneration job. C49 alone permits one interruption and one designed resume, at most two restarts. 100 API operations and 900 seconds; target one fresh retrieval before/after. A search retry counts; no indexing/polling loop may exceed this deadline/operation cap. |

Overlapping classes take the **stricter applicable bound per dimension**; this intentionally makes missing/wrong prerequisites BLOCKED rather than granting extra calls. Matching E19–E24 include the selected classifier's real work inside the 8-dispatch ceiling; a WASM component must be prepared externally before the attempt. Embedding use outside C48/C49 also stays under 20 requests / 64 KiB of submitted input per attempt and the original cumulative $10 per provider. Rates/current remaining credit must be bound before a paid call; absent usage is UNKNOWN, never invented as zero.

A per-request stream may wait at most 120 seconds; owned startup at most 180 seconds, both inside the scenario's elapsed ceiling. Polls consume the operation count; bounded backoff can observe state but cannot retry the scenario or dispatch another model call. Exact timeout values go into the selected script when it is authored. A required outcome not reached within these limits is a recorded TIMEOUT/FAIL; missing admitted preconditions are BLOCKED. These bounds do not promise wall-clock completion of the entire catalog.

## Boundaries, execution pattern and completion

1. Select the next single ID. Prefer C33 onward unless the operator chooses another user goal. Write only its feature, thin Node steps and explicit config. Reuse existing helpers/data, not earlier acceptance invocations. Browser steps use the existing Playwright-BDD pattern. No new blanket runner, installs or test platform is required to author this catalog.
2. Before its execution, bind actual fixture prerequisites, supported route/operation, intended small inference/output allowance and owned effects. A source pointer is not proof an operation works. A missing fixture blocks only that ID. Existing configured model/key names are external setup, never values copied into the test.
3. Run only the selected scenario once initially. Capture immutable raw input/output/events plus its product oracle. Return PASS only for the complete declared result and completed owned cleanup; distinguish product FAIL, fixture/measurement FAIL and BLOCKED. No automatic retry. Discuss a demonstrated failure and the immediate-fix-versus-backlog choice before further repairs/reruns.
4. After an approved correction, keep the first failure and rerun only the affected scenario. Never weaken assertions, swap models just to pass, count reasoning as output, fabricate artifacts, or award a pass to configuration labels. A new provider binding needs real positive evidence, not a copied model catalog entry.
5. At PASS update this catalog row/platform cell, add a bounded B-row only for a newly evidenced behavior, link immutable report/raw receipt and append the Mac/Karpathy logs. Stop for operator direction and leave UAR running. A request count, hook, source review or cleanup step never creates another passing scenario.
6. Preserve historical accepted receipts; rerun them only when the operator explicitly selects relevant regression after a changed dependency/path. Historical evidence is not perpetual current-build certification. A final release assessment must disclose source cohorts and unresolved impact rather than replay the entire suite or pretend earlier binaries equal the current tree.

**Planning exit:** every B01–B35 maps once to the 32 historical workflow rows, all 97 old IDs have a disposition, all 70 new IDs have prerequisites/actions/observable results/evidence/ownership, count arithmetic is consistent, and an independent artifact critic has reviewed the catalog. This planning task finishes at the reviewed catalog, not at product validation. No tests, builds, services or signing/phase-exit actions are authorized by it.

**Product coverage exit after future adoption/execution:** every required row for the explicitly selected platform/backend has PASS with its actual source-bound receipt. FAIL/BLOCKED/deferred cells remain visible. A core-only pass may be called common-workflow coverage, never full supported-options/client coverage. Unsupported platform operations require production-capability evidence and an explicit disposition. Historical supporting obligations (main acceptance, issue #426 integration, profile builds, specs/reviews and their 14-obligation ledger) remain separate release evidence; this BDD catalog neither runs them nor silently closes/discards them.

**Explicit residual scope:** arbitrary provider/model combinations, full default/custom/client Cartesian parity, dedicated default-only UI, every below-threshold context/restart combination, durable-write fault injection, load/race/reboot/soak testing, every settings/admin screen and deployment/security certification remain unverified or deferred here. This makes the proposal finite. Any decision to advertise them as validated requires naming the added scenario/cell; this catalog does not prove every designed function merely by reaching its current denominator.

## Maintenance

[Behavior status and historical receipts](behavior-status.md), [coverage audit](coverage-audit-2026-10-27.md), and [Karpathy process](../knowledge/karpathy/manual-bdd-evolution-01a16d22/process.md) remain the evidence trail. This file is the proposed forward scenario denominator. Do not alter historical raw reports to fit the plan. Add/revise a scenario only for an advertised promise, explicit user requirement or observed failure; document the denominator change and rationale. No CI product testing or publication is introduced.

## Independent design review and delivery

The [artifact-only review](bdd-validation-catalog-review.md) closed all three initial MEDIUM findings after source-backed fixture, policy-oracle and resource-bound revisions. Final verdict: APPROVE the revised **proposed catalog**, no required corrections. It confirmed 102 definitions / 32 historical credits / 70 planned, the 22 common gaps, the 71 stateful parity definitions and all 97 legacy dispositions. This approves documentation quality only; the operator has not adopted a new full-suite requirement and no product case has run under this design.

Files authored for this planning task: this catalog, two source inventories and the independent review. Maintained status now links the forward denominator; the existing OpenSpec evidence delta/tasks and scoped phase-plan appendix record design completion without marking old runner/fixture tasks complete. Karpathy/session/decision history is appended. No new guards or runtime/source changes, tests, builds, services, spend, signing, publication or phase exit were introduced. The next action is an operator-selected single scenario, not an automatic sweep.
