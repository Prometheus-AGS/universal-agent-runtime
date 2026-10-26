# Mac behavior coverage — evidence reconciliation, 2026-10-26

**9 validated behaviors across 6 user journeys.** The latest TEAM-01 journey passed on 2026-10-26; earlier receipts remain historical evidence. This records the agreed behavior split without converting partial legacy cases into full passes.

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
