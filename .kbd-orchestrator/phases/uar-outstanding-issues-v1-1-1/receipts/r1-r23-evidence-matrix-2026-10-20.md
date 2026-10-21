# R1–R23 evidence matrix across three binary identities (2026-10-20)

Owner: uar-lead (Opus 5.5). Reconciled from `r-ledger-reconciliation-2026-10-19.md` (all addenda) and the receipts it
cites. Every grade is bound to the binary that produced it. **No grade transfers between identities.**

## Identities

| Id | Binary / profile | Source | Interfaces exercised | Scope of evidence |
|---|---|---|---|---|
| **v4** (candidate of record) | `universal-agent-runtime` release, sha256 `96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154` | pre-session tree (exact build command not recorded; server-full inferred) | standard runtime: OpenAI chat, KB/RAG, agents, settings, MCP admin, runs, AG-UI/A2A refusals; anonymous no-login loopback; 1907 embedded / 1908 remote / disposable fixtures | all R-row grades of record |
| **f1c2f473** | `universal-agent-runtime` dev (debug), sha256 `f1c2f473da31cf903e868ca87965ee37967ea116549ea80741b5d65d21ad8f73` | HEAD `38f0b5d8` + dirty tree, sha256(git diff HEAD) `54512d82…8b64` (masking patch + R21 tool-budget fix + pre-existing dirty work) | standard runtime, anonymous loopback, mock provider | R21 only |
| **a38c3169** | `uar-sidecar` dev (debug), sha256 `a38c3169f776e633b77e667b00c3597888f389a9eacc6d862e3f6e0117bf7335` | HEAD `38f0b5d8` + dirty tree, sha256(git diff HEAD) `326d8783…6f6f` (= f1c2f473 source + launch-bound identity change) | sidecar-only host authentication (launch token + bound principal), mock provider; no A2A gRPC listener; sidecar-forced env | R4, R23 (and sub-evidence for R20) |

Why grades do not transfer:
- v4 versus the debug builds: different source (the R21 fix and the identity change touch the request-budget and auth paths)
  and a different profile.
- f1c2f473 versus a38c3169: a different binary (`uar-sidecar` forces env such as skill evolution off and JWT off by
  default, binds loopback only, has no gRPC) and a different authentication model (host launch token versus anonymous).
- The sidecar is a narrower product surface. It is evidence for the host-bound local profile only, never for the standard
  runtime interfaces.

## Row matrix

Legend: PASS(A) = artifact-bound functional proof; `*` = flagged for operator review; — = no evidence on that identity.

| Row | v4 (record) | f1c2f473 | a38c3169 sidecar | Fresh proof needed for a new candidate of record? |
|---|---|---|---|---|
| R1 | PASS(A) | — | — | yes (standard binary) |
| R2 | PARTIAL: (a)(b)(c leg1) PASS; (c) leg2 UNVERIFIED; (d) FAIL (AG-UI anonymous 401) | — | — | yes; (c) leg2 reachable anonymously by source (plain `agent_id` lane); (d) needs a verified principal |
| R3 | INCOMPLETE (memory_save Required approval; anonymous approval channel closed) | — | — (memory + bound-owner approvals now exist on this lane; not run) | yes; candidate lane = sidecar host-bound |
| R4 | PARTIAL (mechanics PASS under the synthetic JWT; no-login memory refused) | — | **PASS(A)*** (mock reply; launch-bound subject; fresh store; P1 persisted-row overlay) | yes on the new sidecar binary |
| R5 | PASS(A) | — | — | yes |
| R6 | PASS(A) | — | — | yes |
| R7 | FAIL (`/v1/messages` anonymous -> `credential_identity_required`) | — | — (the local-host credential principal may satisfy it; not run) | yes; sidecar lane candidate |
| R8 | FAIL (A2A gRPC unauthenticated) | — | not applicable (no gRPC listener in sidecar mode) | yes; standard binary needs a verified principal; remains a product gap |
| R9 | FAIL (AG-UI family, as R2(d)) | — | — (not run) | yes; sidecar lane candidate |
| R10 | PARTIAL (render + declaration PASS; callback `continue_run` 403 skill_forbidden anonymous) | — | — (not run) | yes; sidecar lane candidate for the callback leg |
| R11 | PARTIAL (agent + MCP legs PASS; no tool-definition REST surface) | — | — | operator scope decision |
| R12 | PASS(A) | — | — | yes |
| R13 | FAIL (multimodal recorded gap) | — | — | yes; anonymous upload path untraced |
| R14 | PASS(A) | — | — | yes |
| R15 | INCOMPLETE (remote storage legs PASS on 1908; turn leg blocked by the 8192 default) | — | — | yes; needs an approved disposable remote SurrealDB |
| R16 | PASS(A) | — | — | yes |
| R17 | PASS(A) | — | — | yes |
| R18 | PASS(A)* (anonymous v4 direct capture; mock reply) | — | — | yes |
| R19 | PASS(A) | — | — | yes |
| R20 | PARTIAL (KB/docs/agents/deletion under the synthetic JWT; run/event durability anonymous; chat history host-owned) | — | sub-evidence: owner-partitioned memory survives a true restart (M3c) | yes; the complete no-login criterion needs memory under host-bound identity plus the object legs on one binary |
| R21 | FAIL (corrected: 5588 + 1000 > 5919) | PARTIAL (T1-T6 PASS; memory leg unexercisable) | — (memory leg now exercisable here; not run) | yes |
| R22 | PARTIAL (input+reserve refusal PASS; tool/child/root legs unverified) | — | — (spawn now available; not run) | yes; tool leg likely anonymous; child/root on sidecar lane |
| R23 | BLOCKED (no verified no-login principal) | — | **PASS(A)*** (2 children joined, 1 cancelled with no later effects, concurrent same-agent-type isolation; mock; bound-owner approvals) | yes on the new sidecar binary |

**v4 verdict of record (unchanged): 9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED — NOT READY.**

## Smallest coherent candidate-of-record path

1. **One frozen source identity** containing:
   - the masking patch;
   - the R21 tool-budget fix (with the pump revert);
   - the launch-bound sidecar identity;
   - the E0308 test-fixture repair;
   - the pre-existing dirty work the phase already depends on.
   The open phase implementation items (`tasks.md`: 6.2, 7.1-7.3) are verification/publication, not code.
2. **Both binaries from that one source and one profile:** `universal-agent-runtime` for the standard runtime interfaces, and
   `uar-sidecar` for the host-bound local profile. The candidate of record is the source identity plus both binary hashes.
   v4 was a release build, so a release-profile candidate is a Tier 3 build: it needs explicit operator approval of the
   literal commands (debug builds remain for proof iteration).
3. **Re-proof on the new binaries:**
   - every v4 PASS row on the standard binary, reusing the existing harnesses (R1, R5, R6, R12, R14, R16-R19);
   - R21 on the standard binary;
   - R4 and R23 (and the R20 memory leg) on the sidecar.
4. **An operator ruling for the no-login rows whose only blocker is the verified principal** (R3, R7, R9, R10 callback, R2(d),
   R22 child/root): may they be graded on the sidecar host-bound lane, as accepted for R4/R23? If yes, each becomes a bounded
   sidecar check. If no, they remain contract gaps on the standard binary.

## Bounded next checks (debug a38c3169 sidecar unless noted; each a runtime lane needing a lane go; no new build)

- R3 working loop: memory_save via a mock tool call, approved by the bound owner -> record written -> memory_recall returns it.
- R7: `POST /v1/messages` through the sidecar as the bound owner.
- R10 callback leg: A2UI action -> `continue_run` as the bound owner -> observed registry state change.
- R2(d)/R9: AG-UI client stream as the bound owner, capturing activation chunks during the run.
- R22 child/root legs: spawn under a child/root budget; refusal before the action and aggregated child usage.
- R21 memory leg: rerun the R21 probe on the sidecar with memory on (fixture-declared window).
- Standard binary f1c2f473 (anonymous): R2(c) leg2 via the plain `agent_id` lane; R22 tool leg via the ReadOnly `echo` tool.

## 2026-10-20T02:59:17Z — R9 repair run + final host-bound-lane grades on sidecar `a38c3169…7335`

R9 repair (1/1, fixture-only): `ROWS_B_ONLY=R9` adds a reconnect GET and a second POST. Attempt 1 is preserved read-only in
`rows-b/attempt1/`. Run 02:57:56Z -> 02:58:41Z; hash checked; disposable store deleted.
- First POST stream: `RUN_STARTED` -> `RUN_ERROR projection_unavailable` at id `…:6:99` (no text, no RUN_FINISHED).
- **Reconnect** `GET /api/uar/ag-ui/agents/rb-agent/threads/{thr}/runs/{run}/stream`: 200 SSE with RUN_STARTED,
  TEXT_MESSAGE_START, CONTENT x2, END, RUN_FINISHED(success); messageId `<runId>:assistant`; reply
  `RB-REPLY:AGENT-MARK-RBAGENT…`; monotonic ids (0,0)…(14,100). **All R9 criteria pass on the replay.**
- Second POST (new ids): `RUN_ERROR projection_unavailable` again at `…:6:99`.
- Diagnosis (Explore, Sonnet 5.5 requested; 12 observed; source-inferred): the `observe` task (`agui.rs:200,301-309`) seals
  the durable row, so replay succeeds, while the live `stream` copy's `reconcile` fails and the error is discarded
  (`agui.rs:351` `Err(_)`, with no log). The race hypothesis is unconfirmed; the failure is reproducible (3/3 live POSTs
  across 2 boots). The POST path's owner key is computed once and used for both write and read (`agui.rs:47,85,265`), so
  host-bound identity is not implicated by source, but that is not proven at runtime.

**Demonstrated product defect:** the live AG-UI POST stream on embedded SurrealKV emits `RUN_ERROR projection_unavailable`
instead of the run's chunks, while the durable projection completes (replay is correct). The real error is swallowed
without logging.

**Final candidate-specific grades on `a38c3169` (host-bound sidecar lane, operator ruling 02:41Z), all flagged (mock model):**
- R3: **PASS (A)\***. Configured `native_echo`: trace, tool result upstream, reply from the result.
- R4: **PASS (A)\*** (02:15Z).
- R7: **PASS (A)\***. `/v1/messages` 200; the reply reflects the system-prompt agent. The route has no agent selector.
- R9: **FAIL (demonstrated defect).** The live POST stream fails 3/3; the reconnect replay meets all R9 criteria.
- R2(d): **FAIL (product gap).** Automatic activation works (model input + native `skill.activated`), but the AG-UI
  projector drops `SkillActivated` (`agui.rs` ~232-244), so there is no activation chunk on the AG-UI stream. The R9 defect
  also applies. R2 as a row stays PARTIAL; (a)/(b)/(c) leg1 evidence is v4-only.
- R10: **PASS (A)\***. Render with identity, durable replay, declared action -> 202 continued, state 0->1, callback delivered,
  undeclared/wrong-surface 422.
- R22: **PARTIAL.** The root tool-call class PASSES. The child class and aggregation are not demonstrated. Gap: budget-admission
  refusals report no class to the client.
- R23: **PASS (A)\*** (02:15Z).

**v4 tally of record unchanged: 9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED — NOT READY.**
These candidate grades do not transfer to v4 or to the standard runtime binary.

## 2026-10-20T03:07:28Z — R20 complete proof on sidecar `a38c3169…7335`: **PASS (A)\***

Owner: uar-state (Sonnet 5.5 inherited; 4 observed). Run 03:04:49Z -> 03:06:26Z. Boot b1 03:04:52Z and b2 03:05:53Z, hash
a38c3169… checked per boot. True restart: stdin-EOF graceful stop, exit 0 after 31 s, same store. Bound alice; memory ON (source:
launch env + yaml, no persisted row); disposable store deleted; X1 PASS on both boots. Receipts: `uar-r14/rows-r20/`.
- **P1:** KB present; doc A indexed (1 chunk) with a search-A hit; agent r20-keep identical; 3 memory records and the policy
  equal; 3 run records + event histories byte-identical.
- **P2:** doc B absent (search-B 0 hits; control had 1 before delete); r20-gone 404/absent; deleted memory record
  `capture-3ce69d85…` absent (DELETE /api/memory/records/{id} -> 200).
- **P3:** post sets equal pre (kb 1, docs 1, agents 6, memory 3), no duplicates; no phantom active runs; random run id 404/404.
- **P4:** the post-restart S1 question injects the record inside `[MEMORY CONTEXT]`; reply `RECALL:r20-a0bbe74741dc`.
Lead check: 14 of 16 snapshot hashes are identical pre/post. The 2 differences are expected: the in-process `runs.json`
active list (3 -> [], the no-phantom evidence) and the pre-only `search-B-control.json`.
Flags: mock; "threads" = durable run records + event history (lead ruling 22:31:46Z; chat-session prompt history is
host-owned).
This meets the operator's R20 condition (complete no-login/restart criterion on one binary) for candidate a38c3169 only.
**v4 R20 remains PARTIAL** (v4 tally of record unchanged: 9/6/5/2/0/1, NOT READY).

Candidate a38c3169 summary:
- PASS\*: R3, R4, R7, R10, R20, R23.
- PARTIAL: R22 (root tool-call class PASS; child/aggregation not demonstrated; class not reported).
- FAIL: R9 (live stream), R2(d) (projector gap).
Next: the queued R9 source fix (`agui.rs:351`, single writer = R9 owner uar-runtime, source only). Its build + rerun wait for
the literal-command instruction recorded at 03:06:07Z.

## 2026-10-20T08:36:06Z — R9 diagnostic build + one R9 run (approved): mechanism established; correction; grades

**Build** (approved literal command; cached target/; lead-run): 08:31:14Z -> 08:33:55Z, **exit 0**, 2m40s, **0 warnings / 0
errors** (the diagnostics patch compiles). New sidecar binary
**`8e99277b319abc9b6f73cb36c5af8b5182ff0816504774df229de1bb08eec813`** (`uar-r14/r9diag/uar-sidecar-r9diag.bin`, read-only),
source-scoped identity `1b45978c…6a4e85` (= approved source; agui.rs ddb837a4). Behaviour is identical to a38c3169 except for
diagnostics.
**Run** (`ROWS_B_ONLY=R9` once, after a fixture-only repoint of BIN/expected sha; prior R9 receipts preserved read-only in
`rows-b/attempt2/`): 08:34:41Z -> 08:35:27Z, hash 8e99277b… checked, **21 PASS / 0 FAIL**:
- the first live POST and the second live POST both stream RUN_STARTED, TEXT_MESSAGE_START/CONTENT/END and RUN_FINISHED(success),
  with messageId `<runId>:assistant`, the reply reflecting the agent, and monotonic ids (0,0)…(13,100);
- the reconnect is identical.
**Diagnostics observed:** 0 stream-side warnings. One `observe`-side warning at 08:34:53Z: "AG-UI terminal projection remains
pending", error **"agui reconcile: seal"** (outermost label; observe logs `%error`).
**Mechanism (established):** `seal` returns **Err** (not CAS `Ok(false)`) when `observe` and `stream` seal the same `agui_runs` row
concurrently (`compare_and_swap_agui_run` `.check()?` on SurrealKV, `agui_runs/surreal.rs:41-42`). Whichever copy loses gets
the Err. On a38c3169 the stream lost 3/3 and aborted with `projection_unavailable`; here the observer lost (it simply retries)
and the stream completed. Outcome order depends on timing.
**Correction (lead error):** the 03:01:23Z claim that the cursor-6 position "falsifies the seal-race hypothesis" was WRONG. The
error frame's cursor is the stream's local projection cursor, while `reconcile` reads the run's full history from `start_cursor`
and can reach the terminal event and call `seal` before the stream projects past 6. The superseded claim is marked; this entry
replaces it.
**Grades:**
- **R9 on a38c3169: FAIL** (unchanged).
- **R9 on 8e99277b: PARTIAL.** The frozen R9 criteria are met on this run (2/2 live POSTs + reconnect). The concurrent-seal Err
  race is demonstrated, and the behaviour-identical code failed 3/3 on a38c3169, so a single passing run is not a reliable PASS.
- The causal fix is now identified: make a losing concurrent `seal` non-fatal. For example, treat the SurrealKV write-conflict
  Err from the CAS as `Ok(false)` and reload, matching the existing loser path at `agui.rs:259-260`. Do this only for the
  specific conflict error, confirmed from the full chain on the next run. A further source edit, build and run need a new
  literal instruction.
Receipts: `uar-r14/r9diag/` (build), `uar-r14/rows-b/` (run logs `rows-b-r9diag.log`, stderr/stdout). v4 tally of record unchanged
(9/6/5/2/0/1, NOT READY). Count ~1887/2232.

## 2026-10-20T09:01:54Z — R9 fix build + probes: new sidecar candidate `3325e9dd`; R9 PASS (A)\* there

Build (approved; cached; lead-run): 08:53:39Z -> 08:56:16Z, **exit 0**, 2m36s, **0 warnings / 0 errors** (the typed
`surrealdb::types::QueryError::TransactionConflict` match compiles). **New sidecar candidate:**
`3325e9ddf52f8bacd4c95273421a4cb3b92c57e279b4a520de93a16b4be6728d` (`uar-r14/r9fix/uar-sidecar-r9fix.bin`, read-only). Source-scoped
identity `629eea07…` (agui.rs 801499db, agui_runs/surreal.rs f27797ee); HEAD 38f0b5d8.
Fixture-only changes to rows-b.sh: BIN/expected repoint to r9fix; `RUST_LOG=info,…agui_runs=debug` in the sidecar env. Prior run
receipts are preserved read-only in `rows-b/attempt3/` (8e99277b run) and `attempt4/` (first r9fix run).
- **Approved run** (08:57:23Z -> 08:58:09Z): 21 PASS / 0 FAIL. First POST, reconnect and second POST all RUN_FINISHED(success), ordered
  chunks, agent reply, monotonic ids.
- **Bounded repeat x4** (routine focused verification under the 08:56:19Z directive; same probe): 4 x 21 PASS / 0 FAIL.
- **Total:** 5 boots, **10/10 live POST streams + 5/5 reconnects PASS**; 0 stream/observer warnings; 0 "lost a write race" lines.
  RUST_LOG debug lines were 0 in all logs, so the debug filter's effect is unconfirmed.
**R9 on 3325e9dd: PASS (A)\*.** Flags:
- mock;
- the conflict-mapping path was not observed to fire (the race did not recur in 5 boots), so the fix's efficacy is inferred from the
  typed mapping, the earlier observed seal Err on 8e99277b, and the absence of failures; it is not directly observed;
- the race has not been deterministically reproduced.
History: a38c3169 R9 FAIL (3/3 live failures); 8e99277b R9 PARTIAL (1 run, observer seal Err). The a38c3169 receipts for other rows
do **not** transfer to 3325e9dd.
Next (autonomous, routine causal repair): R2(d). The AG-UI projector drops `SkillActivated` (`agui.rs` ~232-244), while
`adapters.rs:134` already maps it to a CUSTOM `uar.skill.activated`. Same file, same single writer (R9 owner). Then build + an
R2(d)+R9 focused rerun on the resulting candidate. Count ~1907/2232.

## 2026-10-20T09:16Z — R2(d) fix build + rows-b on new sidecar candidate `96e5b1f6`: R7, R9, R2(d), R10 PASS\* there

Build (autonomous, directive 08:56:19Z; cached debug sidecar bin, same command as the 3325e9dd build; lead-run): 09:03:52Z -> 09:06:24Z,
**exit 0**, 0 warnings / 0 errors. **New sidecar candidate:** `96e5b1f66c78a663eca0ef672fc5244ba39e5496ccc1edfa62acb22e74d5f5d7`
(`uar-r14/r2dfix/uar-sidecar-r2dfix.bin`, read-only; `r2dfix/expected.sha256`). Source-scoped identity `3a874787…` (agui.rs 90cc1b4c;
surreal.rs unchanged from 629eea07); HEAD 38f0b5d8. The only source delta from 3325e9dd is the `SkillActivated` projector arm.
Fixture-only: rows-b.sh BIN/expected repointed r9fix -> r2dfix (backup `rows-b-pre-r2dfix.sh.bak`; rows-b.sh sha256 48e4c801…,
assert_rows_b.py 17e19ace…). Prior outputs preserved read-only in `rows-b/attempt5/`.
- **Full run** (default mode, 09:08:38Z -> 09:09:54Z, boot sha256 matched): `rows-b/rows-b-r2dfix.log`, **28 PASS / 0 FAIL**, EXIT 0.
- **R9 bounded repeat x5** (`ROWS_B_ONLY=R9`, same probe): `rows-b/r2dfix-r9reps/rep1-5.log`, 5 x 21 PASS / 0 FAIL; each boot
  sha256 = 96e5b1f6. Every first POST, reconnect and second POST ended RUN_FINISHED(success).
- **R9 totals on 96e5b1f6:** 11/11 live POST streams + 5/5 reconnects PASS. stderr had 0 TransactionConflict / "agui reconcile"
  lines; the only WARNs were the known embedding-unavailable, missing-skills-dir and shutdown live-query-kill warnings.

Grades — **candidate 96e5b1f6 only, host-bound sidecar lane (operator ruling), mock LLM:**
- **R2(d) PASS\*.**
  - Matching prompt: the alpha overlay is in the model input; the AG-UI stream carries CUSTOM `uar.skill.activated` (idx 1, before
    RUN_FINISHED idx 5) with `runId` and `agentId`.
  - Control prompt: no activation and no CUSTOM frame.
  - Was FAIL on a38c3169 (no stream chunk).
- **R9 PASS (A)\*.** The flags are the same as on 3325e9dd:
  - the conflict-mapping path was again not observed to fire;
  - the race has not been deterministically reproduced.
- **R7 PASS\*.** Anthropic messages route; the reply reflects the system marker and differs by system prompt; upstream captured.
  - Carried caveat: the route has no agent selector, so the "agent" is the client system prompt. It is UNCONFIRMED whether the
    frozen row intends registry-agent resolution.
- **R10 PASS\*.** Registry builtins; render triggered; durable replay; live chunk with runId+surfaceId; action accepted (202) and the
  state changed before/after; callback delivered to the agent with the continuation reply; undeclared and wrong-surface actions
  refused 422 with no op published.
  - Carried caveat: registry "semantics" beyond the builtin id list are UNCONFIRMED.
No transfer: a38c3169 / 3325e9dd receipts for any row are not 96e5b1f6 evidence, and vice versa. The **v4 tally of record is
unchanged: 9/6/5/2/0/1, NOT READY.**
Rows still not closed on any sidecar identity: R22 child/aggregation (PARTIAL on a38c3169), R21 memory leg (PARTIAL on f1c2f473 main),
and R4/R20/R3/R23, which are graded only on a38c3169. To consolidate onto one sidecar identity, the a38c3169-only rows
must be re-run on 96e5b1f6. Count ~1924/2232.

<!-- 2026-10-20T10:14:54Z -->
## CHECKPOINT — operator reboot checkpoint (Travis, 10:05:34Z): one full row on c7, then pause

Operator instruction, recorded verbatim in substance:
- finish exactly ONE more full frozen verification on an identified candidate, then preserve everything and pause;
- do not grade R22 PASS and do not continue R22 repairs;
- reconcile R3, including the required `builtin_db_registration` test, and grade it only if the whole criterion is met;
- otherwise, take one other bounded row (R21 suggested);
- the scoped autonomy authorization stands, and the per-command approval gate is not reinstated.

**Candidate c7:** sidecar `48d3052320a9a10f4f130488f289042f31e69a47810a3f1a4504a990d5fdc84a` (`uar-r14/c7/uar-sidecar-c7.bin`,
read-only). Source-scoped identity `d1f632813e521a33e1163f037037e0d907f807539eb864967553c696855af401`; HEAD 38f0b5d8.
- Build: 10:03:10Z -> 10:05:35Z, exit 0, 0 warnings.
- Delta over c5: R22 class reporting (control.rs, cost_budget.rs `client_failure`, orchestrator.rs) + the storage diagnostics.

**R22 on c7 (final run, 10:06:46Z -> 10:07:45Z; `rows-a/rows-a-c7.log`): NOT GRADED (operator), FAILs stand.**
- PASS: R22-R1/R2 (root tool-call class); C2/C3 (child limit enforced before the action; 4 children, 5th refused); AGG0.
- C1 is FAIL by the harness regex ("concurrent children"). The tool result now reads
  `agent_tree_limit_exceeded: max_concurrent_children`, and the regex does not match that wording. Not regraded.
- AGG1 FAIL: the budgeted parent's 2nd admission was refused with `budget_admission_failed: accounting_budget_exhausted`
  (client-visible) before the child ran.
  - Fixture arithmetic: the 16000-token catalog liability for parent and child together exceeds the 30000 limit.
  - Aggregation is not demonstrated. A 40000-limit fixture was designed but not run (operator stop).
- An earlier c6 run of the same chain logged `other_storage_error` instead. It did not recur on c7 (storage_error_detail `none`).
  Intermittent and unidentified.
- Earlier-cycle harness hazard (lead): the 10:05:46Z c7 attempt FATAL-exited on an inline api_key (503 credential storage) and left
  its sidecar running. The lead stopped that sidecar; receipts are in `rows-a/run-c7-fatal/`.

**R3 reconciliation on c7: NOT GRADED — the criterion is not fully met.**
- Harness legs PASS on c7: R3-P0 (grant persisted), A1 (echo tool call in the run trace), A2 (tool result in the follow-up
  upstream request), A3 (reply `TOOL:<token>` decided from the tool result).
- Frozen command part 2, `tests/builtin_db_registration.rs`: it did not compile on this tree (pre-existing; the ModelConfig literal
  lacks `execution_profile`).
  - Repair allowance used: a one-line test-only fix (`execution_profile: None`; test file sha256 044a8adb…).
  - Then, run under the operator's routine authorization, 10:09:35Z -> 10:12:21Z, exit 101:
    `cargo test --locked --no-default-features --features server-full --test builtin_db_registration`. Receipt:
    `uar-r14/c7/r3-test/test.log`.
  - It failed to build an unrelated `[[bin]]` that cargo builds for every integration test:
    `tools/seed-minimax-verification/main.rs` (UNTRACKED, another writer's file), E0382 at :225 and :239 ("borrow of moved value:
    record_id").
  - Not touched (not this lane's file; out of scope). The required test therefore did not run, and R3 stays at its prior grades.
- Mapping caveat: by its header, this test file asserts builtin-skill DB registration (R1 content), not tool calling.

**R21 on c7: PASS (A)\*** — the one full bounded row of this checkpoint.
- Run 10:12:53Z -> 10:14:02Z, `bash ./r21sc-probe.sh </dev/null`. Boot sha256 = c7. 31 s. Launch token absent; store and home
  deleted.
- **16/16 PASS** (leg A, the literal criterion, plus leg B). F=3251, X=460, W=8211.
- System, RAG, both skills and memory preserved. Sliding-window trim, oldest first: H3-H5 survive; SEED, H1, H2 dropped.
- The memory block (codeword) is present in every turn after trimming starts.
- Final ~6789 + 1000 reserve = 7789 <= 8211; dispatched.
- Reply `SOURCE:<kb> RECALL:<memory>`.
- Receipts: `r21-sidecar/run4.console.txt`, `out/`. The c5 run-3 receipts are preserved in `r21-sidecar/run3-c5/`.
- Flags: mock; chars/4 estimates (chars/3 = 9052 would exceed W); the memory cap uses the static default profile.

**Grades of record:** v4 tally unchanged, **9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED — NOT READY.**
Candidate-specific (no transfer):

| Candidate | Grades |
|---|---|
| a38c3169 | R3 PASS\*, R4 PASS\*, R7 PASS\*, R10 PASS\*, R20 PASS\*, R23 PASS\*; R22 PARTIAL; R9 FAIL; R2(d) FAIL |
| 3325e9dd | R9 PASS(A)\* |
| 96e5b1f6 | R2(d) PASS\*, R9 PASS(A)\*, R7 PASS\*, R10 PASS\*; R21 FAIL |
| ecd39865 (c5) | R21 PASS(A)\*; R22 PARTIAL |
| 66ccf1dc (c6) | R22 not graded |
| 48d30523 (c7) | R21 PASS(A)\*; R22 not graded (FAILs C1, AGG1); R3 not graded |

**Remaining blockers:**
1. R22 aggregation not demonstrated, and the C1 harness wording mismatch.
2. Product finding for an operator decision: no production path attaches `budget_contract`, so token budgets are unusable on
   custom-provider models. Fixing it is a trust decision.
3. The intermittent non-conflict SurrealDB admission error (c6), unidentified.
4. R3's frozen test target is blocked by another writer's untracked `tools/seed-minimax-verification` bin.
5. No single sidecar identity yet carries all rows. Re-proofs of R3/R4/R7/R9/R10/R20/R23/R2(d) on the final candidate are pending.
6. Earlier open rows on v4 (R8, R11, R13, R15, R23 v4 BLOCKED, etc.) per the matrix.
7. Not committed. The tree is dirty, about 531 status entries, including other writers' work.

**Uncommitted source changes made by this team** (all listed for the operator):
- `src/uar/api/agui.rs`
- `src/uar/persistence/providers/agui_runs/surreal.rs`
- `src/uar/memory/context_builder.rs`
- `src/uar/runtime/cost_budget.rs`
- `src/uar/runtime/thread/control.rs`
- `src/llm/orchestrator.rs` (plus the earlier R21 fix)
- the identity files
- the E0308 fixtures
- `tests/builtin_db_registration.rs` (1 line)

**Quiescence (lead-checked after the R21 run):**
- No uar-sidecar, mock, approver, harness, cargo or rustc process from this team is running.
- Ports 1916/1917/1919, 18916/18917/18919 and 50156/50157/50159 are free.
- Teammates are idle: uar-state (R21 harness), uar-runtime (cost_budget/control/orchestrator), Explore (finished).
- The only listening runtimes are the pre-existing 1906 (pid 32578), 1907 (24103) and 1908 (26153), untouched as instructed.
- **Reboot safety:** safe for this lane, with no active run, build or writer. A reboot WILL terminate 1906/1907/1908, which this
  lane was told to leave untouched; their owner decides.
- Uncommitted work lives only in this checkout on /Volumes/T7. Scratch receipts are under the macOS per-user temp dir
  (/private/var/folders/.../T/opencode/uar-r14), which a reboot may clean. Copy that directory before rebooting if the receipts must
  persist; the ledger excerpts here do not replace them.
Cumulative calls: ~2066/2532 (extension #11 recorded at the c6 entry).
