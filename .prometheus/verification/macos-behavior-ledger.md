# Mac behavior coverage — evidence reconciliation, 2026-10-26

**33 validated behaviors across 30 user journeys.** The latest KB-UNLINK-ONE-01 journey passed on 2026-10-26; earlier receipts remain historical evidence. This records the agreed behavior split without converting partial legacy cases into full passes.

## Confirmed behavior ledger

| ID | Verified behavior | Evidence |
|---|---|---|
| B01 | Prompt configured default agent | [prompt](2026-10-26T06-38-40.440Z-prompt-macos.json) |
| B02 | Receive a useful incremental streamed answer | [stream](2026-10-26T06-56-36.601Z-streaming-macos.json) |
| B03 | Create, persist and query a custom mathematics agent | [math](agents/math-specialist-db64c2a8.verified.json) |
| B04 | Persist a writing specialist and route a writing request to it by intent | [writer](agents/writing-specialist.verified.json) |
| B05 | Create a knowledge base | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B06 | Add and genuinely index a document in a knowledge base | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B07 | Persist a knowledge-base reference on an existing agent | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B08 | Answer using retrieved KB content with matching source provenance | [kb](knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json) |
| B09 | Execute math and writing team members concurrently, then merge both real outputs into an article | [TEAM-01 PASS](team-01-macos-pass.json) |
| B10 | Continue a saved math-agent conversation using server-held history without client transcript replay | [CONTINUITY-01 PASS](continuity-01-macos-pass.json) |
| B11 | Prevent deletion of default and orchestrator agents while preserving their definitions | [PROTECTED-AGENTS-01 PASS](protected-agents-01-macos-pass.json) |
| B12 | Update custom-agent instructions, observe changed inference behavior, and delete the disposable agent | [AGENT-LIFECYCLE-01 PASS](agent-lifecycle-01-macos-pass.json) |
| B13 | Prompt the saved math agent through the Anthropic-compatible Messages API | [ANTHROPIC-01 PASS](anthropic-01-macos-pass.json) |
| B14 | Prompt the saved math agent through A2A v1 and receive a completed task with its answer | [A2A-01 PASS](a2a-01-macos-pass.json) |
| B15 | Prompt the saved math agent through ACP and receive its correct answer with completed run status | [ACP-01 PASS](acp-01-macos-pass.json) |
| B16 | Prompt the saved math agent through AG-UI and assemble a correct streamed answer with correlated successful completion | [AGUI-01 PASS](agui-01-macos-pass.json) |
| B17 | Receive a genuinely published A2UI greeting surface and matching state updates through AG-UI agui_spec stream mode | [A2UI-01 PASS](a2ui-01-macos-pass.json) |
| B18 | Render A2UI controls in Chrome and deliver a fresh user action to the agent for a correct continuation answer | [A2UI-BROWSER-01 PASS](a2ui-browser-01-macos-pass.json) |
| B19 | Implicitly detect and activate the matching keyword skill, then answer using its private recipe and marker | [SKILL-DETECTION-01 PASS](skill-detection-01-macos-pass.json) |
| B20 | Select and use the matching skill from a paraphrased prompt with zero trigger matches, using local embeddings at threshold0.65 | [SEMANTIC-SKILL-01 PASS](semantic-skill-01-macos-pass.json) |
| B21 | Reduce oversized context within the token allowance while preserving original facts for a follow-up | [CONTEXT-BUDGET-01 PASS](context-budget-01-macos-pass.json) |
| B22 | Generate a summary of the fact-bearing conversation turn and preserve its key facts for a follow-up | [CONVERSATION-SUMMARY-01 PASS](conversation-summary-01-macos-pass.json) |
| B23 | Preserve a conversation across a graceful UAR restart and recall earlier facts without replay | [CONVERSATION-RESTART-01 PASS](conversation-restart-01-macos-pass.json) |
| B24 | Retain both overlapping same-conversation turns across a graceful restart and recall their facts | [CONCURRENT-CONVERSATION-RESTART-01 PASS](concurrent-conversation-restart-01-macos-pass.json) |
| B25 | Update a user setting through REST and retain it across a graceful UAR restart | [REST-SETTING-RESTART-01 PASS](rest-setting-restart-01-macos-pass.json) |
| B26 | Use the saved math agent’s retained knowledge base after UAR restart with matching retrieval evidence | [KNOWLEDGE-RESTART-01 PASS](knowledge-restart-01-macos-pass.json) |
| B27 | Change embedding model/dimension, reject the incompatible index, regenerate the retained document and retrieve its original private fact | [KNOWLEDGE-REGENERATION-01 PASS](knowledge-regeneration-01-macos-pass.json) |
| B28 | Delete an indexed KB document so it disappears from retrieval while another document in the same KB remains searchable | [KNOWLEDGE-DOCUMENT-DELETE-01 PASS](knowledge-document-delete-01-macos-pass.json) |
| B29 | Delete a linked KB while its unchanged agent still answers in a fresh conversation without the deleted knowledge or citations | [LINKED-KB-DELETE-01 PASS](linked-kb-delete-01-macos-pass.json) |
| B30 | Unlink a KB so the agent stops using it in a fresh conversation while the unchanged KB remains directly searchable | [KB-UNLINK-01 PASS](kb-unlink-01-macos-pass.json) |
| B31 | Relink an existing KB to an agent and restore grounded use of its unchanged document in a fresh conversation | [KB-RELINK-01 PASS](kb-relink-01-macos-pass.json) |
| B32 | Link two KBs to an agent and combine private facts from both with matching source citations | [TWO-KB-ANSWER-01 PASS](two-kb-answer-01-macos-pass.json) |
| B33 | Unlink one of two KBs so only the remaining source contributes facts/citations while both KBs remain searchable | [KB-UNLINK-ONE-01 PASS](kb-unlink-one-01-macos-pass.json) |

## Additional coverage found in the existing receipts

These are additional covered assertions within the same journeys, not newly executed scenarios or an increased behavior count.

| Original case | What is already evidenced | Still unverified in that case |
|---|---|---|
| U01.configured-prompt | Useful genuine configured-model prompt | Legacy repeated/backend configuration/correlation contract not fully re-executed |
| U03.custom-agent | Create/read/list, restart/reload, distinctive saved instructions; later PATCH/read KB binding | Custom deletion and complete revision/update lifecycle not exercised |
| U06.live-roster-intent | Two registered specialists; intent selects writer rather than math; selected writer executes | Roster update/removal, ambiguous/unmatched intent and changed-roster rerouting not exercised |
| U07.official-agui-selection-replay | AG-UI selected-agent event precedes answer; correlated thread/run and successful terminal | Browser rendering, reconnect/replay and follow-up continuity not exercised |
| U05.openai.json.default | OpenAI-compatible nonstream default-agent response | Two-turn context recall and complete original repetition/backend predicates not exercised |
| U05.openai.json.custom | OpenAI-compatible nonstream saved custom-agent response | Two-turn context recall, custom deletion and complete original parity predicates not exercised |
| U05.openai.openai.default | OpenAI streaming content chunks, useful assembly, stop and DONE | Two-turn context recall and complete original repetition/backend predicates not exercised |
| U14.knowledge-lifecycle | Create/upload/index, persisted binding, real retrieval, grounded answer with exact KB/document/seal citation | Document update/deletion, retained-vs-deleted facts, KB deletion and lifecycle recovery not exercised |
| U20.effective-configuration | Supporting setup persisted/activated FastEmbed settings; live provenance and real ingestion agree | Second chat-model precedence, failed-write controls and config reload after restart not exercised |
| U21.startup-maintenance | Actual UAR restart followed by retained agent reload/use | Empty-store seed uniqueness, named maintenance, migrations/idempotence and preservation of all operator settings not exercised |
| U22.native-tool-approval | Normal spawn_agent interrupt, explicit approval, AG-UI resume, selected writer result and successful terminal | Approval denial/nonexecution, cancellation, unknown effects and broader native tool lifecycle not exercised |

## Count and limits

The original manifest contains 97 bundled cases. Existing receipts overlap **11 case IDs**, each marked `OBSERVED_SUBSET` in the [machine-readable ledger](macos-behavior-ledger-2026-10-26.json); the remaining 86 receive no credit from these five journeys. This is not 11/97 fully passed, nor 8/97: behavior labels and bundled legacy cases have different boundaries. The original multi-backend/repetition contract was not run. Preserve all earlier independent-sweep results separately; this document does not erase them.

The extra confirmed observations include actual agent reload after restart, default/custom OpenAI nonstream prompting, OpenAI streaming, AG-UI selection delivery before answer content, and a successful approve/resume path for the requested native spawn_agent operation. Shared receipts are linked above; none requires a new run to establish that it already happened.

REST embedding configuration activation is supporting evidence: real FastEmbed provenance and indexed ingestion confirm effectiveness, but configuration restart/failure/precedence cases remain unverified. Similarly, a single specialist child thread does not establish agent-team concurrency or multi-call context. The KB response is not evidence for model regeneration, content deletion, or cross-owner isolation.

Known failed attempts remain recorded. The first full persisted-agent scenario failed at response parsing; its creation/restart/read steps passed and the targeted query continuation passed. The writer had setup failures before passing. KB had route/format assertion failures before its passing run. No failed report is relabeled wholesale as passed.

Scope: recorded Mac native artifact SHA256 `25151ae66312e41c99060971645b2414b28817669a164f064cd59d3b0576828c`, built source `sha256:28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018`, real Liter/MiniMax-M3 and disk-backed Surreal. No whole-current-source, other-backend or other-platform certification. The math definition has since gained KB instructions/bindings; its earlier arithmetic result is historical evidence, not a new regression pass on that updated definition. UAR remains untouched and running.

Documentation changes only: this ledger and JSON, appended coverage/phase/session/memory notes, and supported KBD position update. No guards, product changes, tests or unrelated additions.

## 2026-10-26 — Next team journey failed (no new whole-scenario credit)

See [Mac team result](macos-team.md). Team creation, membership and explicit task-board setup passed as steps. Actual member runs failed skill selection; concurrency and merge remain unverified. U17 therefore has additional planning/setup evidence but remains incomplete; U18/U19 receive no new execution credit. The prior8 behavior labels/5 journeys are not changed by this failed sixth journey. Current service PID64463/ownerexec66217 after required identity-configuration restart; historical PIDs above remain historical.

## Operator direction — manual verification supersedes packet workflow
On 2026-10-26 the operator explicitly instructed: "we are manual now and want to totally disregard that method--just keep the list of scenarios." This supersedes the packet/review/inventory/signing/phase-exit workflow for this manual verification effort. Keep scenario tracking and truthful saved results. Run one operator-selected behavior at a time against the persistent UAR, record the result, and stop for direction. Do not require another packet, signature, source review gate, whole-phase inventory or KBD closure ceremony before the selected BDD. Preserve historical receipts without treating their former administrative gates as active requirements. Preserve service/data ownership, secrets, explicit provider spending limits, and the operator's stop-on-failure/no-automatic-retry instruction.

Current selected scenario: TEAM-01 concurrent math/writing work and merged article, still not passed. Packet003 execution had not started when this direction arrived. Its two-file repair is prepared: final-answer parsing plus fixture model output maximum512→64000. Existing compiled Rust binary is unchanged. The prior approval covers the owned restart and this one BDD; it is not permission to run other scenarios. No review/build gate is required. Retain the existing eight validated behavior records; do not rerun them or mark TEAM-01 passed. The last observed service PID is46949; use retained launch metadata for the actual owned restart.

## Returned to manual parent — 2026-10-26
Operator requested leaving this child and running only the existing manual TEAM-01. Owned UAR restart passed (PID50663), preserving store/agents; no build/review gates ran. Command `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs` failed in21.988s:5steps passed(includingAfter),1failed,2skipped. Concurrent member runs and correct math passed; writer output lacked a completed answer outside reasoning. Dependent article/merge was not exercised. Evidence: `.prometheus/verification/team-repair-003-macos.cucumber.json`.

Child work stops here and control returns to `uar-verification-simplification` for manual operator direction. This is an operator-requested return with failed acceptance, not a successful child certification/archive. Keep TEAM-01 FAILED in the scenario list and preserve the eight earlier validated behaviors. Do not automatically repair/retry, resurrect packets, or require signing/inventory/review/exit gates. UAR remains running. No final successful reflection or accepted OpenSpec archive is claimed.

## TEAM-01 passed — 2026-10-26
The existing math/writing team BDD passed on Mac: 1 scenario, 8 steps, 1m 14.458s. Command: `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs`. Tests supplied no authentication credentials; the external endpoint supplies identity. The run proves persistent team creation/membership, distinct overlapping specialist runs, correct arithmetic and completed editorial output, settled accounting with bounded reservations, and a final article consuming both actual artifacts. It adds one team behavior/journey; it does not certify every bundled U17/U18/U19 obligation or another platform. See [receipt](team-01-macos-pass.json), [article](team-01-macos-article.md), and [Cucumber report](manual-team-noauth-003-macos.cucumber.json). Earlier failed attempts remain unchanged. The authenticated-fixture/packet instructions above are historical and superseded by the operator's manual/external-auth direction. UAR remains running; no additional scenario starts.

## CONTINUITY-01 passed — 2026-10-26
The new two-turn math-agent BDD passed on Mac:1scenario,6steps,11.440seconds. A fresh session first added randomized numbers; a separate follow-up contained only references to the previous result/original second operand and a new multiplier. Both final JSON answers matched independently calculated results and the saved specialist identity. Distinct completions returned the same session. No client transcript replay or test credentials. Initial attempt stopped before inference because the model verification was stale; the existing provider verification endpoint refreshed it, and the same scenario then passed. No production code, model selection, provider limits, or authentication configuration changed. No prior scenario reran. Evidence: [receipt](continuity-01-macos-pass.json). This closes warm conversation continuity; restart durability and cross-session memory remain unverified. UAR stays running and execution stops for operator direction.

## PROTECTED-AGENTS-01 passed — 2026-10-26
One scenario passed all5steps on Mac in0.80seconds. Both default-agent and orchestrator-agent were read, DELETE returned HTTP403 with the exact built-in protection message for each, and subsequent GETs returned unchanged complete definitions. This verifies built-in deletion protection, not an unrelated authentication rejection. No model calls, test credentials, restart, previous scenario rerun or production change. [Receipt](protected-agents-01-macos-pass.json). Completed for this manual sequence; UAR stays running and execution stops for operator direction.

## AGENT-LIFECYCLE-01 passed — 2026-10-26
One scenario passed all9steps on Mac in6.580seconds. A new disposable agent followed its original arithmetic rule and random marker; PATCH plus GET confirmed different saved instructions. The same user prompt in a fresh session then produced the updated rule result and new system-only marker. DELETE returned204 and GET returned404. Both attempts' disposable agents were removed. Initial fenced-JSON parsing failure and raw output remain recorded; the parser now uses the existing team BDD's unambiguous final-JSON handling, with unchanged exact answer assertions. No existing agent changes, previous scenario reruns, test credentials, production repair or restart. [Receipt](agent-lifecycle-01-macos-pass.json). UAR stays running; stop for operator direction.

## ANTHROPIC-01 passed — 2026-10-26
The single nonstreaming saved-math Messages scenario passed all4steps on Mac in11.822seconds on its first attempt. POST/v1/messages returned HTTP200, type message, role assistant, the requested liter/MiniMax-M3 model, msg_ identity, text content and end_turn. The final JSON contained the saved specialist marker and independently calculated numeric answer. No model switch, test credentials, agent mutation, restart, production repair or prior scenario rerun. [Receipt](anthropic-01-macos-pass.json). Streaming and full protocol conformance remain unverified by this scenario. UAR stays running; stop for operator direction.

## A2A-01 passed — 2026-10-26
One nonstreaming A2A v1 saved-math scenario passed all4steps on Mac in4.539seconds on its first attempt. SendMessage returned a correlated JSON-RPC success with TASK_STATE_COMPLETED for the selected saved agent, matching task/context message identities, its saved specialist marker and the correct independently calculated arithmetic answer. No test credentials, agent/model changes, restart, production repair or previous scenario rerun. [Receipt](a2a-01-macos-pass.json). Streaming, task continuation/cancellation, legacy A2A, other platforms and full conformance remain outside this result. UAR stays running; stop for operator direction.

## ACP-01 — 2026-10-26

First attempt passed on Mac: 1 scenario, 4 steps, 3.827 seconds. Saved-agent session, streamed answer and completed run identity verified. [Receipt](acp-01-macos-pass.json). No prior scenario reruns; full ACP conformance remains outside this claim.

## AGUI-01 — 2026-10-26

First attempt passed on Mac: 1 scenario, 4 steps, 7.15 seconds. Saved-agent streamed answer, message ordering and successful correlated completion verified. [Receipt](agui-01-macos-pass.json). This directly targets the math agent; prior orchestrator selection and earlier scenarios were not rerun. Full AG-UI/A2UI conformance remains outside this claim.

## A2UI-01 — 2026-10-26

Attempt002 passed: 1 scenario, 4 steps, 6.824 seconds. Native renderer publication, artifact/state agreement, run completion and disposable cleanup verified. Attempt001 parser failure retained; no previous scenario reruns. [Receipt](a2ui-01-macos-pass.json). Scope is agui_spec on chat/completions; dedicated AG-UI agent-route A2UI projection, rendering and actions are not certified.

## A2UI-BROWSER-01 — 2026-10-26T22:53:58.936068+00:00

Attempt007 passed: 1 browser scenario, all5Gherkin steps; test25.4s, Playwright28.4s. Visible calculator, fresh606 input, actual button action202, distinct successful continuation and correct625 plus saved marker in assistant DOM. Disposable deleted; no test credentials or earlier scenario reruns. Fixed native-tool catalog schema and served current frontend; six failed attempts preserved. [Receipt](a2ui-browser-01-macos-pass.json), [details and limits](macos-a2ui-browser.md). UAR remains running; stop for operator direction.

## SKILL-DETECTION-01 — 2026-10-26T22:59:32.061584+00:00

Attempt002 passed1scenario/5reportedsteps in5.655s. Configured keyword strategy selected and implicitly activated only the parcel skill; genuine answer280 and private marker prove use. Decoy unselected; disposable resources deleted. Initial404 setup failure retained. No prior scenario rerun, production/config change, credentials, restart or publication. [Receipt](skill-detection-01-macos-pass.json), [scope](macos-skill-detection.md). UAR remains running; stop.

## SEMANTIC-SKILL-01 — 2026-10-26T23:06:34.193261+00:00

Attempt002 passed: 1 scenario, 5 reported steps, 17.395 seconds. Zero exact triggers; local embeddings at threshold0.65 selected/activated only delivery; its private recipe and marker produced360. Original configuration restored and disposable resources deleted. Initial0.5 threshold admitted gardening too and remains a recorded failure, not a default-quality pass. [Receipt](semantic-skill-01-macos-pass.json), [limits](macos-semantic-skill.md). No previous scenario reruns, production edits, authentication or restart. UAR remains running; stop.

## CONTEXT-BUDGET-01 — 2026-10-26T23:28:14.185251+00:00

Attempt002 passed: 1 scenario / 6 reported steps / 18.413 seconds. History reduced12,869→4,830 tokens within5,879 allowance; final request estimate5,339 and provider input4,689 fit6,144 input budget. Correct original reference and906 total with no fact replay. Initial bundled-message fixture failure retained. Original provider view restored, model readiness refreshed, disposable deleted. [Receipt](context-budget-01-macos-pass.json), [scope](macos-context-budget.md). No prior scenario reruns, production edits, credentials or restart; UAR running, stop.

## CONVERSATION-SUMMARY-01 — 2026-10-26T23:33:52.413377+00:00

Attempt002 passed: 1 scenario / 5 reported steps / 10.430 seconds. Completed summary input digest binds original fact-bearing turn; estimated context3,968→917tokens. Follow-up preserves random booking reference,19attendees andGateQ without replay. Initial receipt-format fixture failure retained; no production changes. Disposable deleted, global settings unchanged, no prior scenario reruns or restart. [Receipt](conversation-summary-01-macos-pass.json), [limits](macos-conversation-summary.md). UAR running; stop.

## CONVERSATION-RESTART-01 — 2026-10-26T23:52:44.642458+00:00

Attempt002 passed: 1 scenario / 6 reported steps / 17.442 seconds. Same store and binary across PID47246→47773; exact random reference and959total recalled without replay. First attempt exposed missing normal-chat save/load and is retained. Host persistence repaired; local Mac build passed. Disposable deleted, UAR running, no prior scenario reruns. Sequential graceful restart only; concurrent same-session durability remains unverified with a source-review risk recorded. [Receipt](conversation-restart-01-macos-pass.json), [scope and repair](macos-conversation-restart.md). Stop.

## CONCURRENT-CONVERSATION-RESTART-01 — 2026-10-27T00:01:47.877656+00:00

First attempt passed: 1 scenario / 6 reported steps / 21.43 seconds. A→B→A server snapshots prove simultaneous running in one session; after PID47773→52394 restart, exact alpha/beta references and933total recalled without replay. No production changes, build or previous scenario reruns. Disposable removed; UAR running. One interleaving only, not exhaustive race elimination. [Receipt](concurrent-conversation-restart-01-macos-pass.json), [scope](macos-concurrent-conversation-restart.md). Full available public conversation and process decisions captured in Karpathy logs. Stop.

## REST-SETTING-RESTART-01 — 2026-10-27T00:07:31.794289+00:00

First attempt passed: 1 scenario / 5 reported steps / 13.834 seconds. REST changed prompt_caching_enabled null→true; full saved record including timestamp survived PID52394→62351 restart. Original null preference restored and read back. No provider calls, product edits, rebuild or earlier scenario reruns. UAR running. Per-user setting persistence only, not caching effectiveness. [Receipt](rest-setting-restart-01-macos-pass.json), [scope](macos-rest-setting-restart.md). Process and public conversation appended to Karpathy logs; stop.

## KNOWLEDGE-RESTART-01 — 2026-10-27T00:18:28.588039+00:00

First attempt passed: 1 scenario / 5 reported steps / 22.95 seconds. Saved math agent, KB and indexed document unchanged across PID62351→73870 restart. Fresh-session answer returned11/199/39601 and document-only seal with actual matching RAG citation. Same local embedding space; no upload, rebinding, reindexing, production edit, build or previous scenario rerun. Existing resources and UAR retained. [Receipt](knowledge-restart-01-macos-pass.json), [scope](macos-knowledge-restart.md). Process and conversation recorded in Karpathy; stop.

## KNOWLEDGE-REGENERATION-01 — 2026-10-26T19:40:34.981660-05:00

First live attempt passed: 1 scenario / 6 reported steps / 7.328 seconds. FastEmbed BGE384→OpenAI text-embedding-3-small1536; old-index409; same retained document regenerated to completion; retrieved coefficient73/private seal with score0.8107894. Disposable KB deleted, original settings and exact boot config restored; UAR PID97354 running. Provider credentials were handled only by the host launcher; none entered the BDD environment or requests. No product edit/build or previous scenario rerun. [Receipt](knowledge-regeneration-01-macos-pass.json), [scope](macos-knowledge-regeneration.md). Karpathy process and public conversation recorded. Stop.

## KNOWLEDGE-DOCUMENT-DELETE-01 — 2026-10-26T19:44:56.458617-05:00

First attempt passed1scenario/5reportedsteps in2.925seconds. Identical query returned both documents before DELETE204 and only retained control afterward; deletedGET404, control metadata unchanged, KBcount1. Owned disposableKB cleaned up. Local embeddings; no credentials/provider spend, settings changes, restart, production edits/build or earlier scenario rerun. [Receipt](knowledge-document-delete-01-macos-pass.json), [scope](macos-knowledge-document-delete.md). Karpathy process/conversation recorded; UAR remains running; stop.

## LINKED-KB-DELETE-01 — 2026-10-26T19:49:08.234441-05:00

First attempt passed1scenario/5reportedsteps in14.145seconds. Before deletion genuine answer476 plus private seal and matchingRAGcitation; after KBDELETE204/GET404, unchanged agent with retainedKBreference answered476 in freshsession with nullseal/nullcitation and zeroRAGcitations. Owned agent/KB cleaned up; no credentials, hosted embeddings, config changes, restart, production edit/build or previous scenario rerun. [Receipt](linked-kb-delete-01-macos-pass.json), [scope](macos-linked-kb-delete.md). Karpathy process/conversation logged; UAR staysrunning; stop.

## KB-UNLINK-01 — 2026-10-26T20:07:08.758403-05:00

Attempt004 passed1scenario/5reportedsteps in21.190seconds. Initialrun exposed implicitAuto access after unlink; fixed shared agent patch persistence to record explicit selection. Two subsequent parser-format failures preserved and corrected without weakening functional assertions. Agent answered679/nullseal/nullcitation after link-onlyPATCH; zeroRAGcitations, KB/doc unchanged, directretrieval stillreturned originalseal. Disposable resources cleaned up. Mac build passed; owned runtime restarted preservingstore/config. No earlier scenario reruns. [Receipt](kb-unlink-01-macos-pass.json), [scope/repair](macos-kb-unlink.md). Karpathy process/conversation logged; UAR running; stop.

## KB-RELINK-01 — 2026-10-26T20:13:44.785282-05:00

First attempt passed1scenario/4reportedsteps in7.925seconds. Restored originalKBID from selected/empty to selected/originalID; freshrealanswer returned correct411 and privatefact with matching originalKB/documentcitation. KB/doc unchanged, no reupload/regeneration. Disposable resources cleaned up; no product/configchanges, build,restart,credentials,hostedembeddings or prior scenario rerun. [Receipt](kb-relink-01-macos-pass.json), [scope](macos-kb-relink.md). Karpathy process/conversation logged; UAR staysrunning; stop.

## TWO-KB-ANSWER-01 — 2026-10-26T20:29:35.070003-05:00

First attempt passed: 1 scenario / 5 reported steps / 8.488 seconds. Genuine answer combined 233 + 864 = 1097, returned each private seal and matching actual KB/document citation. Both saved links and selected policy agreed; no unrelated citations, KB/document snapshots unchanged. Disposable agent and both KBs cleaned up. No credentials, hosted embeddings, global settings changes, production edits/build, restart or earlier scenario rerun. [Receipt](two-kb-answer-01-macos-pass.json), [scope](macos-two-kb-answer.md). Karpathy process/conversation logged; UAR running; stop.
