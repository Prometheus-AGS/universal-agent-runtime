# R1–R23 ledger reconciliation — 2026-10-19 ~17:45 UTC (rev 3, operator-corrected)

Supersedes rev 2 and the intermediate session verdicts where conflicting.
Binding classes unchanged: **A** = artifact-bound functional proof;
**B** = transcript/carry only, not release-grade.

## Whole-row PASS (class A, release-grade) — exactly 4 rows

| Row | Basis |
|-----|-------|
| **R1** | v3 artifact + v4/1907 re-verification (3 runs) |
| **R5** | KB `7c755d7a…` created (201) + attached to r11-probe-agent + readback + empty-KB `{"results":[]}` probe — complete linkage proof (rebind receipt) |
| **R6** | v3 + v4/1907 agent-reflecting replies (incl. "r10 probe ok", 200) |
| **R12** | v4/1907 P2/P3 config-lifecycle receipts + v3; deepened (not re-graded) by the embedding child's v5 provenance/resilience evidence |

## Subcriterion PASS (rows remain partial — never count as row PASS)

- **R2(b)** PASS (A): implicit activation + negatives + disabled-toggle.
- **R2(c) leg1** PASS (A); **leg2 UNVERIFIED** — non-default agent turns
  unreachable anonymous (`model_not_verified`; bound-agent lane requires
  verified principal).
- **R2(a)** PASS (A) as subcriterion: `agents::caveman-help` prefer
  assigned + readback + catalog queryable (rebind receipt).

## FAIL (contract gaps — preserved, truthful refusals)

- **R2(d)**: AG-UI anonymous 401; runs POST skill_forbidden.
- **R7**: `/v1/messages` anonymous → 403 `credential_identity_required`
  (with and without session binding).
- **R8**: A2A gRPC `SendMessage` → `UNAUTHENTICATED verified user context
  required` (compiled-proto probe, r8/r8-result.txt).
- **R9**: same family as R2(d).
- **R13**: multimodal — recorded gap from prior rounds.

R2(d)/R7/R8/R9 are one governance family: on this candidate config only
the OpenAI chat lane serves anonymous turns.

## PARTIAL

- **R10**: registry-backed render + declaration enforcement (422
  negative) proven (A). Callback-driven state change NOT shown: the
  action record was journaled before `continue_run` returned 403
  skill_forbidden; no registry-semantics mutation observed.

## INCOMPLETE (re-bound, explicitly not whole-row)

- **R3**: invocation-in-trace proven (`native__memory_save`,
  `native__memory_recall` with exact args). Working loop NOT proven:
  save refused "Approval channel closed" while the same run replied
  "saved" (contradiction — untruthful-reply finding); recall returned
  `memory_disabled`. All 7 native tools are memory_*; no executable tool
  loop exists anonymous on this config.
- **R4**: conversation recall ("ember-404") + token growth only; the
  required assembled second-turn input was not captured. Memory
  subsystem disabled on candidate config.
- **R11**: agent create (201) + readback (200) + catalog queryable;
  tool-ALLOWLIST readback. Tool-DEFINITION create/readback and
  MCP-server create/readback/reachability NOT done — no REST surface
  (only `/api/uar/mcp/health` + run-scoped interactions).

## UNVERIFIED

- **R14–R20**: re-proof in progress. Done so far on v4/1907 embedded
  SurrealKV: KB `976592bd-4f0c-4b82-948c-1ff873df89a9` created (201),
  doc `44001f56-7a75-428c-9ff7-a412c4c835ac` uploaded (202), KB present
  in list. Vector search with score, representative-flow turn, R18
  source-evidence turn, R19 deletion legs, R15 remote-surreal legs, R20
  reconnect legs all outstanding.
- **R21, R22, R23**: not started.

## Embedding child contribution

No new R-row PASS. R12's class-A binding is deepened (v5 provenance +
restart resilience). Child closed NOT READY (blocked): query-guard leg
UNVERIFIED; live failover NOT MET; external blocker recorded at r3101.

## Call ledger

- Ceiling after extensions #1 (+100→732) and #2 (+200→932): **932**.
- Operator independent counts: 723/732 at 15:14 UTC; 740 at ~15:24 UTC.
- Best reconstruction since 15:24: ~130 calls (child close ~22; R10 ~15;
  R7/R8 ~12; re-binding ~25; R14 start ~3; reconciliation ~3;
  interleaved reads/greps ~50). **Estimated position ~870/932.**
- **Extension #3 (+100 → 1032)** recorded in
  `config-lifecycle-map-2026-10-19.md` before crossing 932, per operator
  instruction 2026-10-19 ~17:45 UTC.

## SECURITY INCIDENT — 2026-10-19 ~17:55 UTC (recorded, value suppressed)

During the R21 settings probe, anonymous `GET /api/uar/settings` on
1907 returned the `llm.api_key` row with the Kimi credential **in
plaintext**. The value was transiently visible in the probe transcript
before being caught; it is deliberately NOT reproduced in any evidence
file, and the scratch copy (`uar-r14/settings.json`) was redacted in
place. Findings, recorded without the value:

1. The candidate's settings store holds the provider credential
   unencrypted at rest (settings row `llm.api_key`, revision
   2026-10-19T09:56Z) despite `CREDENTIAL_ENCRYPTION_KEY` being set for
   this instance.
2. Anonymous (no-login) read of `/api/uar/settings` exposes that row —
   secret material is reachable without any credential on the loopback
   surface.
3. Operator action to consider: rotate the exposed Kimi credential; the
   transcript segment containing it should be treated as compromised.

This incident does not change any R-row grade; it is recorded per the
evidentiary standard and the no-secrets discipline.

---

# Rev 4 addendum — 2026-10-19 ~18:10 UTC (R14–R20 re-proofs complete)

Supersedes the UNVERIFIED entries for R14–R20 in rev 3. Full evidence:
`r14-r20-receipts-2026-10-19.md`.

## Whole-row PASS (class A) — now 9 rows

R1, R5, R6, R12 (rev 3) **+ R14, R16, R17, R19, R20**.

- **R14** PASS (A): embedded-KV representative flow (create/upload/
  retrieval/KB-bound turn) on 1907.
- **R16** PASS (A): create/list/upload/index on 1907.
- **R17** PASS (A): vector search top hit score 0.84012985 recorded.
- **R18** PASS (A) — flagged: PASS on the unlearnable-fact standard
  (reply was the random codeword existing only in the KB chunk);
  assembled input not directly captured. Operator may downgrade.
- **R19** PASS (A): doc delete → 0 direct hits → fresh-agent UNKNOWN →
  KB delete → absent; fixture restored.
- **R20** PASS (A): byte-copy fresh-store boot on hash-verified v4;
  state identical to live, deleted KB absent, no phantom. Method
  caveat recorded.

## Other dispositions

- **R15** INCOMPLETE: storage legs PASS (A) on 1908 remote; turn leg
  blocked by the fixture's 8192 default budget (truthful refusal).
- **R21** UNVERIFIED: trim geometry unreachable within limits
  (1M configured window / security-setting prohibition / spend).
- **R22** PARTIAL: input+reserved-output refusal leg PASS (A) (1908,
  pre-action, exact liability; 500 class noted). Tool/child/root legs
  UNVERIFIED.
- **R23** BLOCKED: run creation anonymous = skill_forbidden; same
  governance family as R2(d)/R7/R8/R9.

## Scoreboard (rev 4)

| Verdict | Rows |
|---|---|
| Whole-row PASS (A) | R1, R5, R6, R12, R14, R16, R17, R18*, R19, R20 (*flagged) |
| Subcriterion PASS | R2(a), R2(b), R2(c) leg1 |
| PARTIAL | R10, R22 |
| INCOMPLETE | R3, R4, R11, R15 |
| FAIL (contract gap) | R2(d), R7, R8, R9, R13 |
| UNVERIFIED | R2(c) leg2, R21 |
| BLOCKED | R23 |

Fixture cleanup: r20 copied volume removed (regenerable); all other
fixtures, probe scripts, and receipts retained. 1907/1908 untouched on
v4; v4 remains candidate of record.

---

## Addendum 2026-10-19T17:58Z — R11 correction + call-ledger correction (operator-directed)

### R11 ledger claim corrected: MCP admin REST routes DO exist

The rev3/rev4 ledger claimed no MCP REST route exists. That claim is **false**, caused by an incomplete grep (route defined via `.nest()` indirection, not a literal `.route("/api/uar/mcp...")` string).

Source evidence (v4/v5 shared line refs, current checkout):
- `src/server.rs:2247-2250` — `.nest("/api/uar/mcp", uar::api::mcp_admin::build_router().with_state(state.clone()))`
- `src/server.rs:1345` — `mcp_admin::hydrate_registry(&mcp, mgr)` at boot
- `src/uar/api/mcp_admin.rs:107-108` — `.route("/servers", get(list_servers))` and `.route("/servers/{name}", put(save_server).delete(delete_server))`

Resulting full paths: `GET /api/uar/mcp/servers`, `PUT /api/uar/mcp/servers/{name}`, `DELETE /api/uar/mcp/servers/{name}`.

Live evidence (port 1907, v4, anonymous, 2026-10-19T17:58Z): `GET /api/uar/mcp/servers` → **HTTP 200**, body shape `{"servers": ...}`; no secret-like strings in body (filtered before inspection). Receipt body retained at `uar-r14/r11-mcp-servers.json`.

**R11 disposition unchanged (INCOMPLETE)** — the correction changes the stated reason, not the grade: the frozen criterion requires tool-definition plus MCP-server create/readback/reachability evidence, which was never executed. What changes: the blocker is no longer "no route exists" but "the CRUD legs were not run". R11 is therefore re-gradable with a bounded probe (create server → readback → reachability) if the operator authorizes it; it is no longer blocked by absence of surface.

### Call-ledger correction

Operator independent OpenCode event count: **930 tool calls at 17:38 UTC** (not the ~870 estimate used in the ext-#3 addendum). Extension #3 (+100 → 1032 ceiling) was recorded before the crossing and is confirmed **timely**. Best reconstruction at 17:58Z: ~960/1032. All addendum timestamps from this point use actual `date -u` output; earlier "~17:45"-style approximations are noted as approximations.

### Security repair in progress (separate receipt)

Operator-directed `llm.api_key` `x-sensitive` repair applied to `src/uar/settings/manager.rs` (build_core_schema); targeted patched-code unit tests added in `manager.rs` and `src/uar/api/settings.rs` and running. Full detail: `security-repair-llm-apikey-2026-10-19.md` (this directory). The earlier port-1913 scratch boot used the unpatched v4 binary with the real key in env: process killed, store scrubbed, zero settings GETs issued to it (health + two 422 PUTs only); old-binary endpoint checks are recorded as **non-evidence** for the patch.

## Call-ledger extension #4 — 2026-10-19 (recorded before crossing 1032)

- Operator independent count: **1017/1032 at 18:34 UTC**.
- **Extension #4: +100 → new ceiling 1132.** Recorded now, before the 1032
  crossing, per operator instruction. Consumed count and all prior extension
  history preserved: base 632; ext #1 +100 (→732); ext #2 +200 (→932);
  ext #3 +100 (→1032, timely); ext #4 +100 (→1132). The operator's earlier
  queued no-extension note was withdrawn by the operator.
- Additive allowance-as-needed remains authorized on the existing coding plan.

## Addendum 19:17Z — R11 re-grade + security repair proof complete

- **R11 = PARTIAL** (was INCOMPLETE). Bounded probe on candidate v4
  (disposable loopback fixture, anonymous, 18:29:19–18:29:23Z): agent leg
  PASS (create 201/read-back 200/queryable/update 200/delete 204→404);
  MCP-server leg PASS (create 200/connected true + live stdio handshake/
  queryable/delete). Tool leg NOT SATISFIED under the frozen wording per
  operator ruling (MCP auto-discovery insufficient): full route enumeration
  proves no standalone tool-definition REST create/configure endpoint exists
  (`GET /api/tools` + `POST /api/tools/{name}/execute` only). Gap recorded
  for operator decision. Receipt: `r11-regrade-2026-10-19.md`.
- **Security repair (llm.api_key plaintext exposure) COMPLETE.** Patch:
  `x-sensitive` on the llm.api_key schema property + 3 regression tests;
  7 pre-existing stale-test compile errors classified (zero production
  failures) and minimally repaired. Targeted tests 9/9 PASS. Runtime proof
  on a disposable pre-patch store with synthetic sentinels: schema refresh
  PASS, saved/effective/list masking PASS, placeholder-cycle PASS,
  replacement reflection PASS; functional chat oracle established as
  architecturally non-evidence (row has no live consumer in this build).
  Patched binary: dev-profile sha256 `0ecff6b2…d8d`, source HEAD `38f0b5d8`
  + dirty tree (uncommitted). Receipt:
  `security-repair-llm-apikey-2026-10-19.md`.
- Remaining rows unchanged: R2(c) leg2 and R23 BLOCKED (external
  verified-principal governance), R21 UNVERIFIED (trim geometry unreachable),
  parent config gate blocked on child blocker r3101. No in-scope probe lanes
  remain except operator-authorized ones.

## Call-ledger extension #5 — 2026-10-19 (recorded before crossing 1132)

- Operator independent count: **1110/1132 at 19:33 UTC**.
- **Extension #5: +100 → new ceiling 1232.** Recorded now, before the 1132
  crossing, per operator instruction (bounded additive allowance as needed
  already authorized). Consumed count and all prior extension history
  preserved: base 632; ext #1 +100 (→732); ext #2 +200 (→932); ext #3 +100
  (→1032, timely); ext #4 +100 (→1132, timely); ext #5 +100 (→1232).

## Addendum R3/R4 lane investigation — handoff checkpoint

- `ToolApprovalPolicy::Auto` exists and the chat lane accepts a typed per-turn
  override: `run_policy: {"tool_approval": "auto"}` (`server.rs:5146`,
  `domain/policy.rs:153+`); session-level override via
  `POST /api/uar/sessions/{id}/agent-config` (`tool_approval` at
  `discovery.rs:548`).
- **However**: the Auto fast-path (`manager.rs:6635-6641`) requires
  `approval_class != ApprovalClass::Required`. Native-tool approval class is
  effect-derived (`runtime/native_skill.rs:81-88`): ReadOnly -> NotRequired;
  ExternalMutation / CodeExecution / Unknown -> **Required**. `memory_save`
  mutates memory (ExternalMutation) -> descriptor forces approval regardless
  of run policy.
- Demonstrated-cause chain for the earlier R3 refusal ("Approval channel
  closed" while the run claimed "saved"): descriptor-forced Ask (above) +
  approval channel unavailable to anonymous runs (same governance family as
  the R2(d)/R7/R8/R9 FAILs). A `tool_approval: auto` fixture will NOT close
  R3's working-loop leg; the gate is the anonymous approval channel /
  verified-principal boundary (external, same blocker as R2(c)/R23).
- **R4 remains reachable without the tool**: `ChatCompletionRequest`
  documents `memory_enabled` -> "memory injection and auto-capture"
  (`server.rs:5126-5129`), i.e. the subsystem writes turns to memory without
  a memory_save tool call. Planned lane: disposable v4 fixture with
  `memory.enabled: true`, tool-driving mock LLM on loopback (logs every
  upstream request body = assembled-input capture), turn 1 auto-capture, turn
  2 assert the entry is present in the captured assembled input and the reply
  reflects it. Distinguish entry-provenance (history vs injected block) in the
  capture and report honestly.
- Fixtures already staged: `uar-r14/fixtures/mock_llm.py` (auth-checking mock;
  needs a tool-call state machine for R3-style drives),
  `uar-r14/fixtures/mcp_stdio_fixture.py`. Model-verification prerequisite
  learned this session: `POST /api/uar/providers/{id}/test` writes the
  ModelVerification record (`providers.rs:420-587`); chat lane 422s
  `model_not_verified` without it. Provider creation body: `ProviderConfig`
  (`llm/registry.rs:33+`).

## Addendum 2026-10-19T21:17:15Z — handoff accepted by Claude (Opus 5.5); agent team activated; call-ledger extension #6

- Packet `bb6363dd-fb55-44e7-bed1-13d2d565b2b5` accepted via `handoff-accept`
  after checks: state rev 48 / task rev 1 matched; all 10 captured sha256
  identities re-hashed equal; HEAD `38f0b5d8` dirty tree preserved; OpenCode
  idle (no repo writes after 21:09:40Z, opencode-cli CPU <1%). Post-accept:
  state rev 49, task `uar111-ff-qual` rev 2, owner uar-lead/claude, pending.
- v4 preserved binary re-hashed: `96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154` (candidate of record).
- Coding agent: Claude Code on Opus 5.5 (operator request). The Kimi-only rule
  governs UAR product model calls only; Kimi weekly quota exhausted, not retried,
  no product-provider substitution. R4 uses the loopback mock LLM fixture staged
  in this ledger's R3/R4 addendum, declared as a fixture in its receipt.
- Conservative row tally carried forward (operator, supersedes rev-4 scoreboard
  where it conflicts): 8 PASS (R1,R5,R6,R12,R14,R16,R17,R19); 6 PARTIAL (R2,
  R10, R11, R18, R20, R22 — R18/R20 pending direct proof); 4 FAIL (R7,R8,R9,R13);
  3 INCOMPLETE (R3,R4,R15); 1 UNVERIFIED (R21); 1 BLOCKED (R23). Verdict NOT READY.
- Agent team (operator requirement): lead = uar-lead/claude, single owner of every
  row verdict and of the cumulative ledger including child calls. Teammates:
  uar-state (R4 runtime lane, sole runtime mutator, cap 30 calls), uar-trust-tools
  (R23 loopback-identity source investigation, read-only, cap 20), uar-verifier
  (artifact-only R4 review, cap 12).
- Count: 1143 at OpenCode close + ~14 lead calls = ~1157/1232 at 2026-10-19T21:17:15Z.
  Team caps (62) + lead remainder will cross 1232. **Extension #6: +100 → new
  ceiling 1332**, recorded before crossing under the standing additive allowance.
  History preserved: base 632; #1 +100; #2 +200; #3 +100; #4 +100; #5 +100 (→1232); #6 +100 (→1332).

## Addendum 2026-10-19T21:20:49Z — team model routing (operator directive)

| Role | Task | Model (actual) | Source of model |
|---|---|---|---|
| uar-lead (lead) | handoff, all row verdicts, final evidence grading, security/Rust/architecture decisions, cumulative ledger | Claude Opus 5.5 | session model |
| uar-state | R4 auto-capture fixture lane (sole runtime mutator, cap 30) | Sonnet 5.5 | `.claude/agents/uar-state.md` model: sonnet (inherited, not overridden) |
| uar-trust-tools | R23 loopback-identity source investigation (read-only, cap 20) | Opus 5.5 | `.claude/agents/uar-trust-tools.md` model: opus (inherited) |
| uar-verifier | artifact-only R4 review (cap 12) — not yet launched | Haiku 5.5 (launch override; definition default opus) | Agent launcher `model` parameter |

Rules: running agents keep their models; new bounded search / fixture prep /
artifact-only review prefers Haiku 5.5, moderate work Sonnet 5.5; ambiguous or
repeatedly failing child work escalates to the lead rather than retrying on a
cheaper model. No new providers, access or spend; no overlapping builds; one
cumulative ledger (children included). Count ~1165/1332 at 2026-10-19T21:20:49Z.

## Addendum 2026-10-19T21:23:50Z — team roster (real-time, provenance-marked) + R23 teammate result

Model provenance key: **observed** = execution metadata; **inherited** = agent
definition, not overridden at launch; **requested** = for a future launch;
**unknown**. This table supersedes the 21:20:49Z table's "Model (actual)" column,
which conflated inherited with observed.

| Role | Task | Model | Provenance | Rationale | Status |
|---|---|---|---|---|---|
| uar-lead | verdicts, grading, security/Rust/arch decisions, ledger | Opus 5.5 | observed (session metadata) | hard grading + boundary decisions | active |
| uar-state | R4 auto-capture fixture lane | Sonnet 5.5 | inherited (`.claude/agents/uar-state.md` model: sonnet) | moderate fixture work | running |
| uar-trust-tools | R23 loopback-identity source investigation | Opus 5.5 | inherited (`.claude/agents/uar-trust-tools.md` model: opus); completion metadata reports no model | security analysis | **finished** |
| uar-verifier | artifact-only R4 review | Haiku 5.5 | **requested** (launch override; definition default opus) | artifact-only review | not launched |

### R23 teammate result (finished; read-only source inspection, no runtime, no HTTP)

- Evidence: `/private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r23/findings.md` (87 lines).
- Tool calls: **22 observed** in execution metadata (teammate self-reported 20; the ledger uses 22).
- Finding: anonymous refusals are identity failures (`ActorOwner::from_verified_context`
  fails -> `SkillAdmissionError::Forbidden` -> `skill_forbidden`). An existing,
  intended, loopback-only host-authenticated path exists: the supervised
  `uar-sidecar` (launch token on stdin, `Authorization: Bearer`, loopback Host,
  no Origin, `x-uar-principal` -> `local-host` credential principal; refused on
  remote profiles). Classified (a): no cloud/network auth default change. No
  unauthenticated trusted-loopback mode found (c). Anonymous OpenAI-chat turns
  cannot reach spawn/join/cancel: the agent/team tools register only for verified
  owners or bound team attempts, and spawn/send/interrupt are approval-Required.
  Team dispatch/admit also need a synthetic `security.settings_admin_key` sent as `x-uar-admin-key`.
- Lead spot-check: `SIDECAR_PRINCIPAL_HEADER = "x-uar-principal"` at
  `src/uar/security/middleware.rs:15`; `sidecar_guard::enforce` at
  `src/uar/security/sidecar_guard.rs:171`; grep shows no trusted-loopback flag. Consistent.
- Unconfirmed by teammate: the sidecar port flag; `skill_role_allowed` behaviour for
  a principal with no roles; anonymous handling at `server.rs:~5558`.

### Lead reconciliation — R23

**R23 remains BLOCKED** because no spawn/join/cancel/parallel assertion has been executed.
The blocker is **reclassified**: it is not an absent identity path. A sanctioned
loopback host-identity path exists, but no sidecar binary carries candidate
identity. Only `target/debug/uar-sidecar` exists (built 14:08 local, source
identity unrecorded); there is no release sidecar and none is bound to v4
`96bf4868…5154`. Proving R23 needs operator decisions: (1) which sidecar binary
identity the evidence may bind to (building one is a build under the closed gate);
(2) authorization to boot a disposable sidecar fixture on a non-1906/7/8 loopback
port with a synthetic settings_admin_key. No action taken; the lead will not build or boot a sidecar without approval.

Count: 1143 + 23 lead + 22 R23 child = **~1188/1332** at 2026-10-19T21:23:50Z (R4 child still running, not yet counted).

## Addendum 2026-10-19T21:28:58Z — R4 teammate result (uar-state, Sonnet 5.5 inherited) + verifier launched

- Evidence: `/private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r14/r4/` (probe logs, SSE, records, mock-requests.jsonl, r4-analyze.py). Binary sha256 observed in both runs = v4 `96bf4868…5154`.
- Run 1 (21:22:29–21:24:21Z): provider create 503 "Protected provider credential storage is unavailable" (inline synthetic api_key) -> fixture defect; **repair 1/1 used** (api_key removed; loopback base_url permitted without key).
- Run 2 (21:24:43–21:26:34Z): provider 201, /test 200, memory policy PUT 200 (enabled, until_deleted). All three turns HTTP 200 but stream `run_failed` `world_state_budget_exceeded` (11448 > 8192: unknown model id defaulted to 8192; world state includes skills from the inherited real HOME). Records empty; auto-capture runs at RunDone, so it never fired. mock-requests.jsonl: provider test + 3 injection-query embeddings, no turn chat requests.
- Source findings (teammate): auto-capture stores user/assistant messages verbatim (Session/Episodic, `capture-<sha256>` ids) and is gated per owner by `PUT /api/memory/policy`. Memory rejects user_id "anonymous" (`claims.rs` ~198-201). Injection is scoped by session_id + agent_id equality, and injected blocks carry the literal `[MEMORY CONTEXT]`.
- Fixture identity: a locally minted HS256 JWT signed with a synthetic `security.jwt_secret` set only in the disposable r4-config.yaml (`jwt_required` false). The frozen R4 wording ("existing harness loopback") does not require anonymous; the anonymous qualifier came from the lead's task packet (lead error, recorded).
- Gate disclosure: the teammate ran `bash -n` and Python `ast.parse` once on its own scratch fixture scripts. These are syntax checks on fixtures, not product verification, and earn no acceptance credit. Disclosed to the operator per the verification lock.
- Tool calls: 23 self-reported (observed metadata pending). Staged but NOT run: third attempt with the mock model `context_window: 128000`; isolated HOME untested.
- **Lead grade: R4 remains INCOMPLETE.** No R4 sub-assertion was exercised. The frozen repair allowance (1) is consumed, so a further fixture repair needs operator authorization.
- uar-verifier launched 2026-10-19T21:28:58Z: artifact-only R4 review. Model **Haiku 5.5, requested at launch** (definition default opus); observed model unknown until completion metadata. Cap 12.
- Count: ~1188 + 23 R4 child + 3 lead = **~1214/1332** (verifier pending).

## Addendum 2026-10-19T21:33:00Z — uar-verifier result + lead reconciliation of R4

Roster update: uar-state **finished** (25 tool calls observed in completion
metadata; 23 self-reported -> ledger uses 25). uar-verifier **finished**. Model
Haiku 5.5 was requested at launch; completion metadata did not report it, so its
provenance stays **requested (unconfirmed)**. Tool calls: 10 self-reported;
observed figure not yet received (provisional).

Verifier (artifact-only) C1-C7: C1 CONFIRMED (all three turns
world_state_budget_exceeded 11448>8192); C2 CONFIRMED (records empty); C3
CONTRADICTED at file level only (the turn-1 injection-query embedding carries the
codeword; the sole chat request is the provider test with no codeword and no
[MEMORY CONTEXT]); C4 CONFIRMED (credential_storage_unavailable / 503,
boot-run1.log:227-228); C5 CONFIRMED (claims.rs:200 rejects "anonymous"; memory
service/controls/scopes route through for_context); C6 provenance CONFIRMED
(1037 skills from the real HOME skills dir) but magnitude CANNOT-TELL (rendered
skill catalog capped ~8000 chars; 11448 is a Cl100kFallback *estimate*); C7 no
non-loopback destination; no live-key markers.

Lead reconciliation of verifier points:
- C7 API_KEY "36 chars, starts sy" = length of the labelled sentinel
  `synthetic-r4-sentinel-not-a-real-key` passed in the env -i boot -> synthetic. Resolved.
- D5 "silent capture loss" NOT SUPPORTED: the 3 embeddings map 1:1 to the 3 turns'
  injection-query embeddings (turn-1 text, then the turn-2 question twice).
  Auto-capture runs at RunDone and every run failed before RunDone. Lead inference
  from source + artifacts; it is not a separate runtime proof.
- D2 accepted in part. Injection is session+agent scoped by design, so turn 2b
  (new session) is only a negative control. The R4 test is same-session turn 2a
  (frozen "same thread / context"). Provenance must separate the codeword inside
  the `[MEMORY CONTEXT]` system fragment from the codeword in replayed history. A
  pass requires the captured record id/content inside that marked block.
- D1 accepted: the next run must hard-fail without that marked-block evidence (no "|| echo 0").
- D3/D4 accepted: hermetic HOME or a fixture-only skill source; record 11448 as an estimate.
- D6 accepted: the synthetic HMAC secret stays in the disposable fixture only.

**R4 grade: INCOMPLETE (unchanged).** No sub-assertion was exercised. The repair
allowance (1) is consumed. A further run needs operator authorization for a second
fixture-level repair: mock model context_window 128000, hermetic HOME, hard
assertions, marked-block provenance. No product source change.

Count: 1143 + 27 lead + children (R23 22 + R4 25 + verifier 10 provisional) = **~1227/1332** at 2026-10-19T21:33:00Z.
Tally unchanged: 8 PASS / 6 PARTIAL / 4 FAIL / 3 INCOMPLETE / 1 UNVERIFIED / 1 BLOCKED. NOT READY.

## Addendum 2026-10-19T21:40:46Z — R4 second fixture repair authorized; run 3 launched

- Operator authorization (chat, verbatim): "Authorized: one more R4 run on v4 with
  the second fixture repair (mock context_window 128000, hermetic HOME/fixture-only
  skills, hard assertions, [MEMORY CONTEXT] provenance separation), synthetic
  fixture JWT owner, no product source change." This is a procedural repair-budget
  item covered by user authorization, not an external blocker.
- Runs 1/2 receipts preserved read-only before rerun: `uar-r14/r4/run1/` (3 files,
  `run1.sha256`) and `uar-r14/r4/run2/` (18 files, `run2.sha256`), chmod a-w.
- uar-verifier observed tool calls = **10** (completion metadata; matches self-report).
- Run 3 assigned to the existing uar-state teammate (resumed, same context; model
  Sonnet 5.5 inherited, unchanged). Outputs go to `uar-r14/r4/run3/`. Hard assertions
  A1 RunDone+chat, A2 record, A3 [MEMORY CONTEXT] codeword in the same-session turn-2
  request with history provenance separated, A4 reply via the memory block only;
  negative control N1 new session. Cap 20. No sensitive-settings GET on v4.
- Count: 1143 + 31 lead + 57 children = **~1231/1332** at 2026-10-19T21:40:46Z (run 3 pending).

## Addendum 2026-10-19T21:41:38Z — roster change

| Role | Task | Model | Provenance | Rationale | Status |
|---|---|---|---|---|---|
| Explore (read-only search) | R21 trim-path source map (cap 10) | Haiku 5.5 | requested (launch override) | bounded search | running |
| uar-state | R4 run 3 | Sonnet 5.5 | inherited | fixture lane (resumed) | running |

Lead hypothesis (not a grade): a mock-provider fixture with a chosen
context_window may reach R21's trim geometry without credential mutation, and the
mock request capture may close R18's assembled-input caveat directly. Runtime lanes
stay serialized after R4 run 3. Count ~1234/1332.

## Addendum 2026-10-19T21:44:21Z — R21 trim-path source map (Explore, Haiku 5.5 requested; 9 calls self-reported)

Refusal (`manager.rs:6022-6053`) applies when world_state_tokens + 1000 >= window
(window = host credential window, else provider model context_window, else catalog,
else 8192). History trim runs only after that check passes: `reduce_history`
(`context/reduce.rs` ~201-280) with input allowance = window - world reserve, a token
budget of window - 1000, pinned system and protected groups. A successful trim emits
`NormalizedEvent::ContextAction` -> AG-UI `agui.context.update` (`api/sse.rs:747-750`).
Overflow of protected history alone yields `protected_history_overflow`/`context_overflow`.
Automatic per run; global `context_strategy` key (`config.rs:321-325`). Lead
assessment: on a mock-provider fixture with world + 1000 < W < world + 1000 + history,
the R21 trim geometry looks reachable with no credential or security-setting
mutation. This supersedes the R21 blocker rationale ("credential-metadata
mutation") for disposable fixtures only. Not yet exercised; R21 remains UNVERIFIED.

## Addendum 2026-10-19T21:47:43Z — R4 run 3 result + lead grade: **R4 PASS (A), flagged**

Run 3 (uar-state, Sonnet 5.5 inherited; 21:44:38–21:45:33Z, 55 s, inside the 60 s
timeout) on v4 `96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154`
(hash checked before boot, in timing.txt and probe-run3.log). Evidence:
`uar-r14/r4/run3/` (assertions.txt, turn1/turn2a-request.json, records-*.json, *.sse,
skills-proof.txt, boot.log). Hermetic HOME + cwd: 0 standard/local/database skills
loaded (was 1037 + 2). Mock model context_window 128000.

Lead independent receipt check (not producer self-report): run1/run2 manifests
3/3 and 18/18 OK. turn2a-request.json has 9 messages; msg 0 (system) contains
`[MEMORY CONTEXT]\n[session/episodic] r4-988d845004f2 is the vault codeword,
remember it` and msg 1 (user) is the replayed-history copy, so the provenances
are separated. Records after turn 1: `capture-6badce5c…` (scope session,
episodic, user, session S1, agent default-agent, content = codeword sentence) plus
the assistant "ACK" record. turn2a reply `RECALL:r4-988d845004f2` (mock decision
source = memory_context only); turn2b (new session) no codeword anywhere ->
NO-RECALL. Cleanup: 1914/18914/50154 free, r4store deleted.

Frozen sub-assertions:
- turn 1 writes a memory entry: **PASS** (auto-capture record, RunDone, no run_failed).
- turn 2 same thread/context assembled input includes that entry: **PASS** (exact captured upstream request, marked injected block).
- reply reflects recall: **PASS at fixture standard**. The reply comes from the loopback
  mock, which answers RECALL only from the injected memory block. This proves runtime
  delivery of the entry to the model and the return path, not real-model reasoning.
- Negative control (new session): PASS, as expected by session+agent scoping.

Flags for operator review: (1) mock-model reply, per the frozen "existing harness
loopback" command; (2) the owner is a fixture-local synthetic JWT, because memory
rejects anonymous owners (`claims.rs:200`) — a recorded product constraint, not
a gap in this row; (3) repairs: 1 frozen + 1 operator-authorized (context_window
128000 + hermetic HOME), both fixture-only, no product source change.
Teammate incident disclosed: a stray `sha256sum` with no args hung on stdin and was
killed; it was a no-op.

**Tally: 9 PASS (R1,R4*,R5,R6,R12,R14,R16,R17,R19) / 6 PARTIAL (R2,R10,R11,R18,R20,R22) /
4 FAIL (R7,R8,R9,R13) / 2 INCOMPLETE (R3,R15) / 1 UNVERIFIED (R21) / 1 BLOCKED (R23).** NOT READY.
Count: 1143 + 37 lead + children (R23 22, R4 25+5, verifier 10, Explore 10) = **~1252/1332** at 2026-10-19T21:47:43Z.
Next (serialized runtime lanes): R18 direct assembled-input capture + R20 real
restart on one disposable v4 fixture (uar-state), then R21 trim on a mock window.

## Addendum 2026-10-19T21:48:31Z — roster change

- uar-state R4 run-3 continuation: **6 tool calls observed** (5 self-reported; ledger uses 6). Explore R21 search: **10 observed** (9 self-reported).
- uar-state (Sonnet 5.5 inherited, resumed) assigned R18 direct assembled-input capture + R20 true restart on one disposable v4 fixture, outputs `uar-r14/r18r20/`, cap 25; it holds the sole runtime slot. R21 trim lane is queued behind it.
- Count: ~1255/1332 at 2026-10-19T21:48:31Z.

## Addendum 2026-10-19T21:51:22Z — operator correction: **R4 = PARTIAL** (supersedes the 21:47:43Z PASS grade)

- Operator ruling: the frozen shared profile is local loopback with **no user login**.
  Run 3 used a synthetic JWT owner, so it proves memory mechanics but not the
  intended unauthenticated, user-facing local flow.
- Preserved unchanged: v4 run-3 receipts `uar-r14/r4/run3/` and the A1-A4/N1 results
  (all PASS at mechanics level: auto-capture record, [MEMORY CONTEXT] in the
  same-session turn-2 assembled input, recall from the block, new-session control).
- **Exact access gap:** memory policy, capture admission, injection search and the
  records API all resolve the owner via `CredentialPrincipal::from_verified_context`,
  which rejects user_id "anonymous" (`src/uar/security/claims.rs:198-201`;
  memory via `for_context` in `memory/service.rs:691-692`, `controls.rs:85-100`,
  `scopes.rs:20`). A no-login loopback caller is `anonymous`
  (`security/middleware.rs:53-105`), so the user-facing no-login flow cannot opt in
  to, capture, or recall memory on the standard server. The candidate's intended
  no-login local identity is the host-authenticated sidecar (launch token +
  `x-uar-principal`), being investigated jointly with R23.
- Clearing condition: runtime proof of the intended local flow on candidate
  identity. The passing mechanics probe will not be rerun without need. No per-route
  auth bypass, no provider/security setting change, no new credentials.
- **Tally: 8 PASS / 7 PARTIAL (R2,R4,R10,R11,R18,R20,R22) / 4 FAIL / 2 INCOMPLETE (R3,R15) /
  1 UNVERIFIED (R21) / 1 BLOCKED (R23).** NOT READY.
- uar-trust-tools resumed (Opus 5.5 inherited, read-only, cap 12): can the v4 binary serve
  the sidecar host-auth flow, and does that path cover memory (R4) and spawn/join/cancel (R23)?
- Count ~1258/1332 at 2026-10-19T21:51:22Z.

## Addendum 2026-10-19T21:55:50Z — R4/R23 local-identity follow-up: **external blocker on candidate identity**

uar-trust-tools follow-up (Opus 5.5 inherited; read-only; 11 calls self-reported)
appended to `uar-r23/findings.md` under "Follow-up: candidate-binary sidecar reachability".
Lead spot-check: `start_server_sidecar` (`src/server.rs:2677`) has exactly one non-test
caller, `src/bin/uar-sidecar.rs:301`; `src/main.rs` has 0 `launch_token` references;
Cargo.toml defines the separate `uar-sidecar` bin (`src/bin/uar-sidecar.rs`). Consistent.

Findings:
1. The v4 candidate (`universal-agent-runtime` main bin, `96bf4868…5154`) **cannot**
   serve the host-authenticated sidecar flow. Main-bin `start_server` passes
   launch_token None, and `x-uar-principal` is refused with `principal_header_not_allowed`
   (`middleware.rs:173-175`). The no-login loopback caller on v4 is anonymous by design.
2. Under the sidecar `local-host` principal, memory would admit the owner by source
   (`claims.rs:221-228`; `memory/service.rs:691-693`; `controls.rs:96-106`), with caveats:
   trusted-host admin branch differences, and an owner key distinct from the JWT run.
3. R23 under the sidecar principal: `skill_required_roles` defaults empty, so there is
   no role-based `skill_forbidden`. The model-facing `spawn_agent`/`wait_agents`/
   `interrupt_agent` path avoids team dispatch and `settings_admin_key`.
4. No document names the launch-token sidecar as the frozen no-login profile;
   `docs/release-verification.md:150` certifies through the packaged binary's
   OpenAI-compatible seam.

Lead reconciliation:
- **R4 stays PARTIAL.** Mechanics are proven (run 3, synthetic JWT owner). The intended
  no-login local flow is unreachable on the candidate binary: anonymous memory is
  rejected (`claims.rs:200`) and the only host-auth identity path lives in a
  different binary. Clearing R4 needs operator/product direction: (a) accept the
  no-login memory refusal as a recorded contract gap (same family as R7/R8);
  (b) bind evidence to a `uar-sidecar` binary built from the candidate source
  (a build under the closed gate, and a second artifact identity); or (c) a product
  change, out of scope.
- **R23 stays BLOCKED.** Same root: the candidate binary has no verified no-login
  principal. Options (a)/(b) as above. No per-route bypass will be added.
- Actual external blocker, reported to the operator; no action taken.

## Call-ledger extension #7 — recorded before crossing 1332

Count ~1272/1332 at 2026-10-19T21:55:50Z (lead + children incl. trust-tools follow-up 11). The queued
R18/R20 (cap 25) + R21 (~20) lanes will cross 1332. **Extension #7: +100 -> 1432**
under the standing additive allowance. History preserved: base 632; #1 +100; #2 +200;
#3 +100; #4 +100; #5 +100; #6 +100 (->1332); #7 +100 (->1432).

## Addendum 2026-10-19T22:02:36Z — R18 + R20 lane result (uar-state, Sonnet 5.5 inherited; 22 calls self-reported) + lead grades

Observed counts: uar-trust-tools follow-up **12** (11 self-reported). Evidence:
`uar-r14/r18r20/` (attempt1/ preserved; graded attempt 2, 21:57:13–21:59:16Z; v4
`96bf4868…5154` checked before both boots; hermetic HOME; mock-only; cleanup verified
by lead: 1914 free, store deleted).

**R18** — mechanics PASS: the exact upstream request (`r18-request.json`) has the
chunk with the codeword once, inside `<uar-retrieved-content>` under `[RELEVANT
KNOWLEDGE]` (msg 0, system, with `[1] (docA.txt)`); the mock reply
`SOURCE:r18-5b18424e5992` was decided only from that block. No SSE citation surface (INFO).
Lead check: **every chat and KB call carried `Authorization: Bearer` (synthetic JWT)**
(`r18r20-probe.sh:44,58`; mock has_auth True on all 4 chat requests). Under the
operator's R4 no-login standard this run does not prove the no-login flow. Unlike memory,
KB is reachable anonymously on v4 (the earlier 1907 anonymous R14/R18). **R18 stays PARTIAL**;
next is an anonymous-only R18 turn with capture (fixture correction, not a blocker).

**R20** — true SIGTERM restart on the same store (not a byte-copy). PASS legs:
KB 1->1, docs 1->1 (doc A), memory 2->2, agents 5->5, equal sets, no duplicates; deleted
doc B and agent r20-throwaway absent, doc-B search 0 hits post-restart; post-restart
S2 memory injection + RAG injection work (`RECALL:r20-f8d9cacb0968`). Not shown:
`GET /api/uar/runs` 2->0 (`list_runs_for_context`, `manager.rs:8366`, reads the
in-process `active_runs` map only, so it cannot show durability); no durable
session/thread list route found (four candidates 404).
Lead finding (from `mock-requests.jsonl`): the post-restart S2 request carries ONLY
the new user message (no replayed pre-restart S2 history). In-process, the
same-session R4 run-3 turn 2a replayed history (msg 1 user). Thread/conversation
history was therefore **not shown to persist across restart**. Cause not diagnosed
(source search launched). Both runs used the synthetic JWT owner (same profile caveat).
**R20 stays PARTIAL**: direct-restart legs for KB/memory/agents/no-resurrection PASS;
thread leg not met.

Probe defects disclosed: attempt-1 R20 `kill -0` liveness defect (repair 1/1 for R20);
transient doc `failed:0` status for ~18 s before indexing in attempt 2, not diagnosed.
Tally unchanged: 8 PASS / 7 PARTIAL / 4 FAIL / 2 INCOMPLETE / 1 UNVERIFIED / 1 BLOCKED. NOT READY.
Count: ~1272 + 1 (trust obs delta) + 22 + 3 lead = **~1298/1432** at 2026-10-19T22:02:36Z.

## Addendum 2026-10-19T22:07:00Z — R20 thread-leg root cause; anonymous R18 attempt 0; new finding

Observed counts: R18/R20 continuation **27** (22 self-reported); ledger uses 27.

**R20 thread leg = contract gap (by design, not a fixture defect).** Source search
(Explore, Haiku 5.5 requested; 8 calls self-reported) plus lead spot-check:
- The chat lane passes only `session_id` to the run manager.
- Turn dialogue lives in-process: `RunDialogue` in `active_runs` (`manager.rs:330,570`) and the
  `SessionStore` map (`manager.rs:588`, `src/session/thread.rs:364+`).
- `get_or_create_for_user` (`thread.rs:443-456`) creates an empty session on a map miss,
  with no rehydrate from the store.
- No durable conversation-message table was found.
- `durable_instances` is derived internally (`persistence/providers/surreal.rs:171`) and
  gates only run-history/accounting records (`run_history/surreal.rs:5-34`), not chat dialogue.
Consequence: chat-lane conversation threads do not survive restart, which matches the observed
post-restart request with no history. **R20 stays PARTIAL**: KB, docs, memory and
agents persist, with no resurrection after a true restart (PASS legs); the thread leg fails as a
recorded product gap. Operator decides whether the contract expects durable threads.

**Anonymous R18 attempt 0** (`uar-r14/r18anon/`, 22:03:27–22:04:17Z, v4 hash checked, no
Authorization on any call): provider create 201, /test 200, KB create 201, doc A upload 202,
doc B upload (sent immediately) **500** `The query was not executed due to a failed
transaction` (boot.log ERROR 22:03:47.287Z, 1 ms). A0/A1/A2 not evaluated.
**New observed finding:** a second document upload while the first doc's ingestion is in
flight can return 500 (failed transaction). Related: in the JWT lane a doc briefly showed
`failed` before indexing. Cause not diagnosed; possibly a write conflict on the
document/job path, not anonymous-specific (unproven). Recorded, no code change.
R18 repair 1/1 (fixture-only, standing operator authorization): single doc A, wait for
`indexed`, fully anonymous. Outputs `r18anon/repair1/`. Last attempt.

Tally unchanged: 8/7/4/2/1/1, NOT READY. Count ~1316/1432 at 2026-10-19T22:07:00Z.

## Addendum 2026-10-19T22:11:33Z — anonymous R18 repair result; lead grade **R18 PASS (A), flagged**

Observed counts: R18-anon attempt-0 continuation **30** in completion metadata (2
self-reported). That figure may include internal steps; the ledger uses the observed
value conservatively. R20-history Explore **13** (8 self-reported). Repair-1 continuation
observed count pending (2 self-reported).

Repair 1 (`uar-r14/r18anon/repair1/`, 22:08:31–22:09:22Z; R18 phase 6 s; v4
`96bf4868…5154` in probe.log): **no client auth anywhere**. probe-auth-grep = 0; the only
match is the audit line itself (`r18anon-probe.sh:61`); the config has no jwt_secret. Steps,
all anonymous: provider 201, /test 200, KB 201, doc A 202, indexed (1 chunk, first poll),
agent PATCH 200, chat 200. R18-A1 PASS: the codeword occurs once, inside
`<uar-retrieved-content>` `[RELEVANT KNOWLEDGE]` (msg 0 system, `[1] (docA.txt)`) of the
captured upstream request `r18-request.json`. R18-A2 PASS: `SOURCE:r18-37383b12e2a9` decided
from the retrieved block only. A0 as written ("mock has_auth False") FAILED because the
runtime's *server-to-provider* call carries its own synthetic provider credential (env
`LLM_API_KEY` sentinel). The lead's A0 wording was wrong: it conflated upstream provider auth
with user login. The no-login criterion (client -> UAR) is met.

**R18 = PASS (A), flagged** for operator review: (1) mock-model reply, decided from the
retrieved block; real-model evidence for the same row exists from the earlier
anonymous 1907 turn (`zephyr-9182`, Kimi, no capture). (2) No citation event on the SSE
stream (the prompt instructs `[1]` citation; the reply reflects the source). (3) Repair 1/1
used (single doc, wait for indexed).

**Tally: 9 PASS (R1,R5,R6,R12,R14,R16,R17,R18*,R19) / 6 PARTIAL (R2,R4,R10,R11,R20,R22) /
4 FAIL (R7,R8,R9,R13) / 2 INCOMPLETE (R3,R15) / 1 UNVERIFIED (R21) / 1 BLOCKED (R23).** NOT READY.
Count ~1357/1432 at 2026-10-19T22:11:33Z. Next: R21 trim lane (uar-state, Sonnet 5.5 inherited, cap 25,
anonymous). The memory component cannot be no-login on v4 (R4 gap); R21's ceiling is therefore PARTIAL.

## Addendum 2026-10-19T22:18:03Z — R21 lane result; lead grade **R21 FAIL (demonstrated)**; call-ledger correction

**Call-ledger correction.** uar-state completion metadata is cumulative across resumes
since run 3 (6 -> 27 -> 30 -> 33). Per-continuation figures: run 3 = 6, R18/R20 = 21,
R18-anon attempt 0 = 3, repair 1 = 3. The 22:02-22:11Z entries overcounted by ~33.
Recomputed: 1143 + 54 lead + children (uar-state 25+6+21+3+3 = 58; uar-trust-tools 22+12;
verifier 10; Explore R21 10; Explore history 13 = 125) + R21 lane (6 self-reported; observed
pending) = **~1328/1432** at 2026-10-19T22:18:03Z. Extension #7 stands, recorded in time; no new extension.

**R21** (uar-state, Sonnet 5.5 inherited; `uar-r14/r21/`, 22:14:37–22:15:35Z, 58 s; v4
`96bf4868…5154` in probe.log; anonymous, no client auth, no jwt_secret; hermetic HOME with
2 fixture skills; KB with doc A attached; mock-only). Calibration refusal on a 1500 window:
`context requires 419 tokens but the input allowance is 117`. Then W = 5919 (second mock
provider, same instance); 6 turns with H1-H5 at ~4400 chars each; Auto -> `sliding_window`.
- T2 system prompt pinned msg 0: PASS. T3 RAG `[RELEVANT KNOWLEDGE]` kept; reply from it: PASS.
  T4 both fixture skills present (msg 0 + permissions msg 9; 15 tool defs): PASS.
- T5 trim order oldest-first, contiguous (final request kept H4, H5 + H6; dropped H1-H3): PASS.
- T6 trim recorded on the OpenAI SSE as `delta.context_updates`, e.g. turn 3
  `{"strategy":"sliding_window","messages_removed":5,"tokens_saved":1788,"was_applied":true,"summary_generated":false}`: PASS.
- **T1 fits real window + reserved output: FAIL.** Lead recomputation on
  `r21-final-request.json` with tiktoken cl100k (the runtime's own fallback encoding):
  messages 3727 + tool definitions (15, 9378 chars) 2391 = **6118 > W 5919**, before the
  1000-token output reserve (~7118 vs 5919). Lead source check: the budget check
  (`manager.rs:6022-6027`) and `context/reduce.rs` have no term for tool-schema tokens
  (grep found none), and the final-wire output check did not reject the request (it reached the
  model). **Demonstrated defect: context budgeting omits tool-definition tokens, so a
  trimmed request can exceed the declared model window.**
- Memory leg not exercised (no-login memory impossible on v4; R4 gap).
- Caveats: the window is fixture-declared (mock), not a real model limit; token counts are
  cl100k estimates. The ~1200-token overflow exceeds plausible estimator drift.
**R21 = FAIL (demonstrated)**, superseding UNVERIFIED. Sub-legs T2-T6 PASS are preserved as evidence.

**Tally: 9 PASS (R1,R5,R6,R12,R14,R16,R17,R18*,R19) / 6 PARTIAL (R2,R4,R10,R11,R20,R22) /
5 FAIL (R7,R8,R9,R13,R21) / 2 INCOMPLETE (R3,R15) / 0 UNVERIFIED / 1 BLOCKED (R23).** NOT READY.

Remaining rows are operator or external decisions only: R4/R23 no-login identity on the candidate
binary (options a/b/c, 21:55:50Z addendum); R3 anonymous approval channel; R15 turn leg
(touches 1908); R10 callback leg and R22 child/root legs (verified-principal family);
R11 tool-definition surface; R2(d)/R7/R8/R9 governance family; R18*/R4 flags; R20
durable-thread contract question; new findings (tool-token budget defect; concurrent-
upload 500). No in-scope runtime lane remains without an operator decision. Runtime slot idle.

## Addendum 2026-10-19T22:18:15Z — R21 observed count

uar-state cumulative counter 33 -> 41: R21 lane = **8 observed** (6 self-reported). Final count **~1331/1432** (1143 + 56 lead + 133 children). Team idle; no running agents; runtime slot idle; 1907/1908 processes untouched; HEAD 38f0b5d8, dirty tree preserved.

## Addendum 2026-10-19T22:24:33Z — repair round (operator directive) + call-ledger extension #8

Operator directive: work demonstrated failed oracles in dependency order. R21: fix only the
tool-definition budget gap (count tool schemas; refuse an unsatisfiable input+reserve before
dispatch), then rerun the focused R21 oracle. R20: read the frozen criterion, inspect the
persisted session/history path, and repair the same-session history gap only if required. R4/R23:
a candidate-local identity design from existing mechanisms. No auth weakening, no per-route
bypass, no new credentials, no unbound sidecar boot without approval. Tally preserved:
9/6/5/2/0/1, NOT READY; R18 anonymous receipt preserved; passing rows not rerun.

**Verification-gate classification (AGENTS.md operator lock 2026-10-17, CLOSED):** source
edits = authorized implementation. Any cargo build/check/test, and therefore any new-candidate
R21/R20 rerun, is verification and needs the operator's explicit instruction naming the exact
commands. That branch will stop for approval while the independent safe work continues.

Count ~1336/1432 at 2026-10-19T22:24:33Z. The planned round (3 teammates, caps 30 + 20 + 15, plus lead) crosses
1432. **Extension #8: +200 -> 1632**, recorded before crossing under the standing additive
allowance. History preserved: base 632; #1 +100; #2 +200; #3 +100; #4 +100; #5 +100; #6 +100;
#7 +100 (->1432); #8 +200 (->1632).

| Role | Task | Model | Provenance | Ownership (single writer) | Status |
|---|---|---|---|---|---|
| uar-lead | grading, diff review, build-approval request, ledger | Opus 5.5 | observed | receipts, ledger, session-log | active |
| uar-runtime (new) | R21 diagnosis + budget fix | Sonnet 5.5 | inherited (`.claude/agents/uar-runtime.md` model: sonnet) | `src/uar/runtime/manager.rs` (run-budget region), `src/uar/runtime/context/**`, `src/uar/runtime/world_state/**` | launching |
| uar-state (fresh) | R20 session/history diagnosis, read-only | Sonnet 5.5 | inherited | none (read-only; writes `uar-r14/r20diag/findings.md`) | launching |
| uar-trust-tools (resumed) | R4/R23 candidate-local identity design, read-only | Opus 5.5 | inherited | none (writes `uar-r23/identity-design.md`) | launching |

## Addendum 2026-10-19T22:31:46Z — R20 diagnosis (uar-state fresh, Sonnet 5.5 inherited; **18 observed**) + lead scope ruling

Evidence: `uar-r14/r20diag/findings.md`. Lead spot-checks:
- `manager.rs:8517-8523`: "Embedded hosts keep their durable conversation history in their own
  store … The in-process SessionStore is not durable" — the host/client seeds history back via
  `SeedMessage`/host_history.
- The OpenAI chat protocol is stateless (the client resends `messages`).
- Restart-durable fallbacks exist in `get_run` (`manager.rs:8263`, fallback ~8236-8250) and
  `history_since` (`:8216`), gated on `supports_durable_run_history` (true for embedded
  surrealkv per findings, `surreal.rs:~157-176,629`). They are reached via
  `api/routes.rs:179` `get_run_for_context` and the `api/agui.rs` history paths.

**Lead ruling:** chat-lane same-session prompt history across a cold restart is host/client-owned
by design, not a runtime persistence obligation. The 21:59Z "no replayed history after restart"
observation is therefore **not a demonstrated defect**, and **no chat-lane repair** will be made
(the proposed save/load_session repair would change a documented design and carries named risks:
delete-race recreation, TTL sweep, loss of world-state host records). R20's "threads" is
interpreted as runtime-durable thread/run records. Flagged for operator review: if the operator
requires runtime-owned chat history durability, that is a product change.

**Affected-leg rerun (no build, no code change, frozen v4):** an anonymous no-login fixture
creates runs in two sessions, does a true SIGTERM restart on the same store, then reads
GET-by-id + the durable event history for each pre-restart run. It asserts no phantom active
runs and that a never-created run id stays absent. KB/docs/memory/agents and deletion
evidence from 21:57Z are preserved, not rerun. Assigned to the same uar-state teammate
(resumed), cap 20, sole runtime slot (the R21 writer edits source only, no builds).
Count ~1366/1632.

## Addendum 2026-10-19T22:32:40Z — gate record for the R20 restart-leg lane

The uar-state teammate correctly paused and asked for the operator authority (0 calls used). Operator instruction
(chat, this session, verbatim excerpt): "R20: first read the frozen restart criterion and inspect
the exact persisted-session/history path. Repair only the demonstrated same-session history gap if
the criterion requires it; preserve the passing persisted objects/deletion evidence, and **rerun only
the affected restart leg**." Lane scope (only this): two boots of `uar-v4-preserved.bin` (sha256
`96bf4868…5154` checked per boot) on 1914/50154 with the loopback mock 18914, anonymous HTTP to the
chat, run-by-id, durable event-history and D5 thread-read routes, a disposable store deleted on exit.
No build, cargo, test or code change. Gate otherwise remains CLOSED; builds still need named-command
approval.

## Addendum 2026-10-19T22:33:34Z — R4/R23 candidate-local identity design (uar-trust-tools, Opus 5.5 inherited; 13 observed)

Design: `uar-r23/identity-design.md` (95 lines; source inspection only). Lead spot-check:
`src/bin/uar-sidecar.rs:191-194` unconditionally sets `UAR_MEMORY__ENABLED=false` and
`UAR_SKILL_EVOLUTION__ENABLED=false` ("Features that mix content across sessions stay off
regardless of configuration files"). Confirmed.

- **R4: no existing mechanism serves the no-login memory flow on any binary.** The main bin needs a
  verified owner (JWT/API key/configured trusted-host issuer); the sidecar forces memory off by
  design; no HTTP capture-policy route was found (MCP tool only, `memory/mcp_server.rs:597`).
  Clearing R4 needs a product change (out of scope) or an operator ruling that accepts the recorded
  gap. **R4 stays PARTIAL; branch stopped.**
- **R23 sidecar design** (not executed):
  - The exact release-profile build command for the uar-sidecar bin is recorded verbatim in the
    design file, section 1. Its features were inferred from the v4 server-full note; the server
    feature dependency is unconfirmed. Tier 3, closed gate, tier-guard hook blocks it; this
    ledger quotes no build text because the hook matches it.
  - Launch/route flow: harness-generated 64-hex launch token on stdin, `--port`, loopback,
    `READY:{port}`; Authorization Bearer token + Host + no Origin + `x-uar-principal`;
    `POST /api/uar/runs` with spawn_agent/wait_agents/interrupt_agent; approvals via
    `/api/uar/runs/{id}/tool-approval`; cancel via interrupt or `/runs/{id}/cancel`; parallel via a
    second run. Avoids team dispatch and settings_admin_key.
  - Security facts: no credential is created, no bypass, no v4/cloud default changes. It is a
    different executable whose launch token lets the host assert any subject; its JWT enforcement
    is off unless JWT_REQUIRED is set; the principal header is refused on remote profiles
    (`middleware.rs:215-217`).
  - Identity: built from the current dirty tree (masking patch + pending R21 fix) = **new
    candidate identity**, not v4.
  - Profile fit: literally no user login, but the subject is harness-asserted, structurally like
    the synthetic-JWT owner ruled PARTIAL.
  **R23 stays BLOCKED; branch stopped pending operator decisions:** (1) approve the exact build
  command; (2) rule whether a harness-asserted host subject satisfies the frozen no-login identity.
- Lead tier-guard note: a lead ledger heredoc quoting that build text was blocked by the hook at
  2026-10-19T22:33:34Z (nothing executed); not bypassed.
- Count ~1385/1632. Running: uar-runtime (R21 source fix), uar-state (R20 restart leg on v4).

## Addendum 2026-10-19T22:34:26Z — R21 source fix authored (uar-runtime, Sonnet 5.5 inherited; **32 observed**); critic launched

Unbuilt, untested, unverified (gate CLOSED). Diff vs HEAD saved: `uar-r14/r21fix/r21-fix.diff`
(sha256 `f5f381b9…`, 1089 lines). Files: `src/uar/runtime/context/reduce.rs` (sha256 `c06364bc…`,
+169) and `src/uar/runtime/manager.rs` (sha256 `6d82d658…`). The manager.rs diff vs HEAD may
include pre-existing dirty-tree hunks; the R21 hunks are the budget region and two activation call sites.
- Diagnosis: tool definitions are rebuilt per orchestrator iteration (`src/llm/orchestrator.rs`
  ~1995-2066) and sit outside `messages`. No budget term counted them. Production
  `budget_contract` is None, so no final-wire check exists. Reconciles 3727 + 2391 = 6118 > 5919.
- Fix: reduce.rs adds `FixedRequestOverflow` (code `context_overflow`), `RESPONSE_RESERVE_TOKENS`,
  `tool_definition_tokens`, `reserve_fixed_request_tokens`. manager.rs moves the authorized-tool
  collection before reduction, refuses before dispatch when world + tools + 1000 >= window,
  trims against window - world - tools (the reserve is still subtracted in reduce_history), and the
  typed ContextStage and skill-activation/orchestrator limits also subtract tool tokens.
  3 tests authored in the reduce.rs test module, NOT run.
- Teammate-named residual risks: mid-run tool drift (search_tools/MCP activation, which needs
  `src/llm/orchestrator.rs`, not owned); skills/team fragments re-attached after reduction are
  unbudgeted; tokenizer is not wire-exact; error ordering changed (turn_assembly_rejected can
  precede context_overflow).
- v4 build command: **not found** as a record. The v5 ledger records
  "--release --features server-full --bin universal-agent-runtime" (v5 attempt 2); v4 is inferred
  to match but this is unconfirmed.
- Independent artifact-only critic (artifact-critic, Sonnet 5.5 requested) launched on the
  diff alone, cap 15.
Count ~1420/1632 at 2026-10-19T22:34:26Z.

## Addendum 2026-10-19T22:37:24Z — R21 critic result (artifact-critic, Sonnet 5.5 requested; **15 observed**) + repair assignment

Findings (artifact-only, no compiler):
- **HIGH:** the admission tool count omits the `search_tools` descriptor inserted when tools are
  deferred (`orchestrator.rs` ~1991-1993), and freezes the tool cost while the orchestrator
  re-projects every iteration (~1894, ~1930), so iteration >1 can reproduce the overflow.
- **MEDIUM:** Typed mode maps ReduceHistoryError to `turn_assembly_rejected` (`builtin.rs` ~254-267),
  losing the code and liability text. The first-turn `reattach_skills` uses the unreduced window
  (manager.rs ~6332). The no-margin token estimate is noted; the lead clarifies that 2391 is a lead
  cl100k measurement (runtime fallback encoding), not a provider figure.
- **LOW:** a production `.expect` (reduce.rs ~49); mostly tautological tests; error-ordering shift.
- Sound: reserve arithmetic (no double subtraction), no underflow, refusal boundary matches the
  existing `+1000 <` convention, pre-dispatch early return mirrors world_state_budget_exceeded
  cleanup, matches exhaustive, toolchain nightly-2026-07-18 (repeat_n available).
- "Scope creep" hunks (RunHistoryWriter, admission_timeout, durable fallbacks, pin_embedding, …)
  are likely pre-existing dirty-tree changes from before this round (diff is vs HEAD). Unverified:
  no pre-round hash of runtime/manager.rs was captured.

Lead disposition: all HIGH/MEDIUM and the production-expect LOW accepted. The uar-runtime writer
was given a single-writer extension to `src/llm/orchestrator.rs` (tool/reduce region) and
`src/uar/runtime/turn/builtin.rs` (ContextStage error mapping), cap 25. No margin added. A second
critic pass follows the repair. Count ~1437/1632 at 2026-10-19T22:37:24Z.

## Addendum 2026-10-19T22:38:04Z — R20 restart leg result (uar-state, Sonnet 5.5 inherited; **11 observed**); lead grade **R20 PASS (A), flagged**

Evidence `uar-r14/r20runs/` (22:34:30–22:35:58Z). Boot 1 22:34:32Z and boot 2 22:35:25Z, v4
`96bf4868…5154` checked per boot. SIGTERM at 22:34:53Z, exit 0 after 30 s, same surrealkv store
(true restart). Fully anonymous (probe Authorization count 0, no jwt_secret). No repair used.
Lead independent check: `cmp` shows pre/post `/api/uar/runs/{id}` JSON identical for S1a, S1b,
S2 and `/runs/{id}/events` identical for all three. Post-restart `GET /api/uar/runs` = `[]`. Random id
-> 404/404. Store deleted, 1914 free.
- D1 run records durable (owner-matched durable fallback, `manager.rs` ~8353-8385): PASS.
- D2 run event history durable: PASS. Presentation note: the retention label stays
  "process-local-bounded" though the events are served from durable history.
- D3 no phantom active runs: PASS. D4 never-created id absent: PASS.
- D5 agent_threads / instance threads: anonymous 401 (`authenticated_owner_required`,
  `verified_principal_required`). Verified-principal family; recorded, no auth added.
- Chat-session context-stats 200 before / 404 after: host-owned history by design (lead ruling 22:31:46Z).

R20 composite: KB, docs, memory and agents persist; deleted doc and agent are not resurrected
across a true restart (21:57Z, preserved per operator); runs and event history persist with no
phantoms (this lane). **R20 = PASS (A), flagged** for operator review: (1) the 21:57Z memory/KB
persistence legs ran under the synthetic JWT owner (memory cannot be no-login on v4; same root
as the R4 ruling); the operator may downgrade on that basis. (2) "threads" is read as durable
run/event records; agent_threads are unreadable anonymously. (3) Chat history across restart is
host-owned by design.

**Tally: 10 PASS (R1,R5,R6,R12,R14,R16,R17,R18*,R19,R20*) / 5 PARTIAL (R2,R4,R10,R11,R22) /
5 FAIL (R7,R8,R9,R13,R21) / 2 INCOMPLETE (R3,R15) / 0 UNVERIFIED / 1 BLOCKED (R23).** NOT READY.
Count ~1450/1632 at 2026-10-19T22:38:04Z. Running: uar-runtime (R21 critic repairs, source only).

## Addendum 2026-10-19T22:42:49Z — R21 fix v2 (uar-runtime; **21 observed**) + second critic launched

Source-only, unbuilt. Diff `uar-r14/r21fix/r21-fix-v2.diff` (sha256 prefix 78f1db00); files and sha256
prefixes: reduce.rs 9c1d33e1, runtime/manager.rs 96aa0c6f, llm/orchestrator.rs dced020d,
turn/builtin.rs 9daa6dc5.
- Design: `activation.context_limit` carries no tool cost. The orchestrator measures the actual
  per-iteration `tools` vector and applies `reserve_fixed_request_tokens` each iteration,
  refusing pre-dispatch with CONTEXT_OVERFLOW.
- Admission counts `search_tools` when deferred.
- Typed path keeps code + liability via tag/split helpers.
- First-turn reattach uses the tool-reduced limit; the production `.expect` is removed.
- Tests rewritten/added, not run.
- Writer-named residuals: iteration-1 orchestrator world=0; string-tagged contributor codes; the
  graph path uses the admission estimate only.
Second artifact-critic (Sonnet 5.5 requested) launched on v2 alone, cap 15. Count ~1474/1632.

## Addendum 2026-10-19T22:48:50Z — second R21 critic (artifact-critic, Sonnet 5.5 requested; **26 self-reported, cap 15 overrun recorded**) + lead triage

Findings on v2 (no compiler):
- HIGH: Typed SkillBodiesStage uses the full window (`builtin.rs:348-351`, `retention.rs:46`).
- MEDIUM:
  - The iteration-1 orchestrator check trusts tool-set parity with admission (world=0, no
    measurement; `orchestrator.rs:1956-1989`).
  - The graph `SkillRequestDriver` uses the admission estimate (`manager.rs:7129`, `usage.rs:97-111`).
  - Code casing differs (orchestrator `CONTEXT_OVERFLOW` vs manager `context_overflow`).
  - The `pump.rs:498-523` allow-list turns Typed overflow into `actor_kernel_failed`, a regression.
  - Tests cover pure helpers only.
- LOW: string-tagged codes; `search_tools_definition` silently under-counts on failure; token
  estimate is not wire-exact.
- Sound: borrows, NativeSkillRegistry visibility, PartialEq, no double subtraction, refusal before
  dispatch with matching termination, liability text.

Lead triage (operator scope: "fix only the demonstrated budget gap"):
- ACCEPTED: the Typed skill-bodies limit, iteration-1 direct measurement, code-family casing, and the
  pump allow-list (a regression introduced by the fix). Assigned to uar-runtime with a single-writer
  extension to `src/uar/runtime/instance/pump.rs` (CODES only), cap 20.
- DEFERRED as recorded residuals (not demonstrated by R21; no evidence-triggered need): graph-path
  per-request counting, an AssemblyError code field, the search_tools fallback, estimator margin, and
  integration tests (need harness work and cannot run under the closed gate).
Count ~1504/1632 at 2026-10-19T22:48:50Z.

## Addendum 2026-10-19T22:52:35Z — R21 fix v3 complete in source (uar-runtime; **12 observed**); approval request filed

Second critic observed **28** calls (cap 15 overrun; recorded). v3 (source-only, unbuilt) closes
the four accepted findings:
- Typed SkillBodiesStage uses window minus tool tokens (`builtin.rs` ~347-358, manager ~6227/6245).
- An iteration-1 measured pre-dispatch check runs on the actual messages + tools
  (`orchestrator.rs` ~2036-2058).
- New refusals emit lowercase `context_overflow`. Pre-existing uppercase codes are left
  (test-pinned `tests/context_history_integrity.rs:562`).
- The pump allow-list gains context_overflow/protected_history_overflow/invalid_history and now
  matches case-insensitively (`pump.rs` ~498-525). Flag: this also preserves other
  uppercase-emitted known codes, a small intended widening.
Residuals recorded: graph SkillRequestDriver counting; destination-preparation tokens after the
iteration-1 measurement; mixed casing at the non-actor API edge for pre-existing codes; deferred
items per the 22:48:50Z triage.

**Approval request filed:** `receipts/r21-build-approval-request-2026-10-19.md`. It gives the exact
build (the v5-recorded command with a dedicated CARGO_TARGET_DIR), the identity record, the focused
R21 rerun, the 15 open implementation items per waypoint (137/152), and what the result would and
would not prove. The R21 branch is stopped pending explicit operator approval. Count ~1524/1632 at 2026-10-19T22:52:35Z.

## Addendum 2026-10-19T23:28:22Z — operator approval: focused R21 debug build + probe; R20 regraded PARTIAL

**Operator approval (chat, verbatim excerpt):** "I explicitly approve the AGENTS.md exception to build
and verify R21 before the whole phase is implementation-complete." Build command as approved; the
operator's en-dashes are read as `--` (typographic conversion), flags `--locked --no-default-features
--features server-full --bin universal-agent-runtime`, debug profile, cached `target/`. Then the
isolated r21-fix probe. Approval covers this focused debug build and probe only, not release or
publication.

**Unfinished items from the current task ledger** (`openspec/changes/uar-outstanding-issues-v1-1-1/tasks.md`:
35 checked, 4 unchecked), recorded once. These are not code defects: 6.2 [V2] local manifest + exact
local invocation; 7.1 [P1] commit/push + v1.1.1 tag; 7.2 [P2] pull published children +
arm64/amd64 integration; 7.3 [P3] GitHub Release. The waypoint's 137/152 count is stale relative to
this ledger and is not treated as 15 defects.

**R20 = PARTIAL** (operator ruling; supersedes the 22:38:04Z PASS). It stays PARTIAL until its
complete no-login/restart criterion is demonstrated. The 22:35Z anonymous run-record/event
durability legs (D1-D4) and the 21:57Z object/deletion legs are preserved as sub-evidence.

**Pump widening resolved:** the case-insensitive match (`eq_ignore_ascii_case`) in `kernel_failure_code`
was REVERTED to exact `code == *known`. Reason: accepted finding #4 needs only the lowercase
`context_overflow`/`protected_history_overflow`/`invalid_history` entries (the new refusals and the
Typed manager path emit lowercase). Case-insensitivity only changed the handling of pre-existing
uppercase orchestrator codes, which no accepted finding covers. The three entries are kept.

**Binary preservation:** `target/debug/universal-agent-runtime` = sha256
`0ecff6b2fbcceb643de09af217e55f7ecbf1835c3543f61998a4c59fa2f64d8d` (the masking-repair proof binary;
also the executable of running instance 1906, pid 32578, which is left running and untouched).
Copied read-only to `uar-r14/preserved-bins/uar-dev-masking-0ecff6b2.bin` (hash verified) before the
build overwrites the on-disk debug artifact. v4 `uar-v4-preserved.bin` untouched.

**Pre-build source identity:** HEAD `38f0b5d85960e3cb8445607750538ca500efdf17`; sha256(`git diff HEAD`) `54512d82a819870792c9b42d688cfa1b23f9e3d3e71f7ea83f6a75372ad88b64`; untracked files
under src/Cargo/build.rs: 4. R21-fix files (sha256 prefix):
  9c1d33e1e16cee7d  src/uar/runtime/context/reduce.rs
  4483820734e6c7ea  src/uar/runtime/manager.rs
  929d76c13e5be42b  src/llm/orchestrator.rs
  f7f3ddad5f04f468  src/uar/runtime/turn/builtin.rs
  8c548f1237f5096d  src/uar/runtime/instance/pump.rs

**Tally: 9 PASS (R1,R5,R6,R12,R14,R16,R17,R18*,R19) / 6 PARTIAL (R2,R4,R10,R11,R20,R22) / 5 FAIL
(R7,R8,R9,R13,R21) / 2 INCOMPLETE (R3,R15) / 0 UNVERIFIED / 1 BLOCKED (R23).** NOT READY.
Count ~1530/1632.

## Addendum 2026-10-19T23:38:59Z — R21 focused build + probe on NEW candidate; v4 measurement correction

**Build** (operator-approved focused debug build, cached `target/`, serial, lead-run): flags
`--locked --no-default-features --features server-full --bin universal-agent-runtime`, dev profile.
23:28:33Z -> 23:34:38Z, **exit 0**, 6m05s, 0 warnings, 0 errors (`uar-r14/r21-fix/build.log`,
`build-timing.txt`). **New candidate identity** (`r21-fix/identity.txt`):
- binary sha256 `f1c2f473da31cf903e868ca87965ee37967ea116549ea80741b5d65d21ad8f73`, copied read-only to
  `r21-fix/uar-r21fix-debug.bin`;
- HEAD `38f0b5d8`; sha256(git diff HEAD) `54512d82…8b64` (unchanged pre/post build); 4 untracked src files
  hashed (run_history postgres/surreal providers, persistence/run_history.rs, rag/embeddings/swappable.rs).
NOT v4. It includes the masking patch, the R21 fix and the other pre-existing dirty-tree work.
Preserved: v4 `uar-v4-preserved.bin`; masking proof `preserved-bins/uar-dev-masking-0ecff6b2.bin`.
Instance 1906 (pid 32578, launched from the replaced target/debug file) is still running, untouched.

**Probe attempt 1** (23:35Z, exit 3; receipts frozen in `r21-fix/attempt1/` + `attempt1.sha256`): the
new pre-dispatch refusal fired on the 1500-token calibration window. `context_overflow`,
"fixed request content needs 3255 tokens (1869 tool definitions, 386 world state, 1000 reserved output)
but the model context is 1500". The mock received **no** calibration chat request (provider /test +
2 embeddings only), proving refusal before dispatch. The probe stopped because its calibration regex
expected the old message. Demonstrated cause: a fixture assumption broken by the intended fix.
**Repair 1/1 (fixture-only, lead):** two-step calibration: F from the fixed-content refusal; X at window
F+1 ("context requires 463 tokens but the input allowance is 1; the allowance already excludes 2254 tokens
of tool definitions and world state and the 1000-token response reserve"); W = F + X + 4500 = 8217.

**Probe attempt 2** (-> 23:37:52Z, **exit 0**; `r21-fix/`; hash checked = f1c2f473…; anonymous;
mock-only; store `r21fixstore` deleted; 1914 free):
- T1 fits: PASS. Lead cl100k on `r21-final-request.json`: messages 5147 + tool definitions 1869 = 7016;
  + 1000 reserve = 8016 <= W 8217.
- T2 system pinned: PASS. T3 RAG block kept, reply `SOURCE:r21-b546d35c59ee` from it: PASS. T4 2 fixture
  skills: PASS.
- T5 oldest-first trim (dropped H1, H2; kept H3-H5 + H6): PASS.
- T6 `context_updates` events at turns 3-6: PASS.
- Pre-dispatch refusal with liability text: PASS (the calibration turns were never sent to the model;
  9 mock chat requests = 3 provider tests + 6 turns).
- Memory leg: not exercised (no-login memory impossible; R4 gap).
- Window is fixture-declared (mock).

**v4 measurement correction (lead error):** the 22:15Z v4 T1 figure "tools 2391, sum 6118 > window" used
spaced JSON re-serialization. Compact (as the runtime serializes): v4 messages 3719 + tools 1869 = 5588,
which fits the window 5919 but **not window minus reserved output** (6588 > 5919). The v4 FAIL stands on
the frozen "fits window + reserved output" wording; the overflow magnitude was overstated.

**Grades.** Row tally stays bound to candidate of record v4: **R21 = FAIL on v4** (unchanged).
On new candidate `f1c2f473…`, R21 is **PARTIAL**: all trim/fit/protection/order/refusal legs PASS; the
memory component is unexercisable no-login; mock window. This is not a v4 PASS, and no other R-row has
been re-proven on the new candidate. Unit tests authored in reduce.rs were NOT run (outside the approval).

Tally (v4): 9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED. NOT READY.
Count: ~1547/1632 (incl. probe-prep teammate 5 observed).

## Addendum 2026-10-19T23:52:18Z — handoff status recorded; read-only continuation started

The R21 approval blocker is recorded as RESOLVED in `.agent-team/uar-core/handoffs/HANDOFF-uar111-ff-qual.md`
(destination addendum). Tally unchanged (v4): 9/6/5/2/0/1, R20 PARTIAL, NOT READY. Read-only round: an
Explore teammate (Haiku 5.5 requested, cap 12) maps whether open rows can advance under already-authorized
lanes. The lead assesses candidate-local host principal binding. Lead source fact:
`src/uar/memory/context_builder.rs:56-60`, injection = "hybrid search over (session, user, agent) then
global", so recall is not session-confined. No code, build, test or boot in this round.

## Addendum 2026-10-19T23:55:42Z — read-only map of advanceable rows + host-principal assessment (no runs)

Explore (Haiku 5.5 requested; 11 self-reported). Source/contract only; nothing executed.

| Row / leg | Candidate | Existing evidence | Shortest permitted next step | New gate approval? |
|---|---|---|---|---|
| R2(c) leg 2 (agent-specific activation, no cross-agent leakage) | v4 `96bf4868…5154` | leg 1 PASS; leg 2 blocked only by model_not_verified (ledger) | v4 mock fixture: verified mock provider, chat with body `agent_id` (not the `uar:agent:` lane, which needs a verified principal: `api/targets.rs:72-74`); capture the upstream body for 2 agents | No build. A runtime lane under the closed gate, so it needs the operator's go for that lane |
| R22 tool leg (refuse before a tool call, report class) | v4 | input+reserve leg PASS (1908) | mock emits a call to ReadOnly `echo` (`native_skills/echo.rs:37`, NotRequired per `native_skill.rs:81`) with a per-turn tool cap in `run_policy` (`cost_budget.rs:432-435`, `policy_intersection.rs:132,181-183`). Tool exposure to anonymous turns is untraced; trace it read-only first | No build. Lane go needed |
| R22 child/root legs | — | — | needs spawn = verified owner + approval (`agents/mod.rs:150,159`; `team_tools.rs:99-106`) | Blocked by the identity decision |
| R13 multimodal | v4 | FAIL "recorded gap" | read-only trace of anonymous `/api/upload` + attachments -> `image_url` (`server.rs:5335-5369`); mock declares vision | Read-only first; then a lane go |
| R15 turn leg | v4 | storage legs PASS (1908) | a disposable SurrealDB (pinned v3.3.0 by digest, `versions.toml:31-34`) on a new port, fresh ns/db (`config.rs:982,985`), mock with a large window; 1908 untouched | **Yes**: starting a new DB process/container is a new operator action |
| R20 object legs anonymous | v4 | object/deletion legs PASS under the JWT owner; run/event legs PASS anonymous | rerun KB/doc/agent/deletion across a restart anonymously (KB uses the anonymous owner key, `knowledge.rs:188-190`) | No build; lane go. Cannot complete R20: the memory leg is the R4 gap |
| R21 on candidate of record | f1c2f473 | PARTIAL (T1-T6) | promoting a new candidate of record and re-certifying the rows is an operator decision | Yes (release-class decision) |
| R3, R4, R7-R10, R2(d), R23 | — | verified-principal / approval family | identity decision | Yes |
| R11 tool surface | — | PARTIAL | operator decision on scope | Decision only |

Host-principal binding (lead assessment; nothing implemented):
- **v4 anonymous (status quo):** memory refused (`claims.rs:200`); threads/instances 401. Coherent by design. R4/R23 = recorded gap.
- **v4 main bin + trusted-host issuer** (`security.trusted_host_principals`, `config.rs:717/798`, `verifier/mod.rs:332`): keeps the v4 binary, but needs a configured issuer and signing key (a new credential plus a security-config change, which is a material security-boundary change). Structurally a host-signed owner, like the synthetic JWT ruled PARTIAL. Enables memory and threads.
- **uar-sidecar** (launch token + `x-uar-principal`): new binary identity; memory **forced off** (`uar-sidecar.rs:191-194`), so R4 is impossible; threads/spawn possible (R23) with a harness-asserted subject.
- **Cross-session isolation risk:** under any single host-asserted subject, injection searches "(session, user, agent) then global" (`memory/context_builder.rs:56-60`). Session-scope captures are filtered by session+agent, but user/agent/global-scope memories cross every session of that owner. A host that maps several end users to one subject leaks across users. Skill evolution mixes similarly. This is the stated reason for the sidecar disable. A coherent binding needs one subject per end user plus session-scoped capture. No existing mode provides it without a product change.
Tally unchanged (v4): 9/6/5/2/0/1, R20 PARTIAL, NOT READY. Count ~1566/1632 at 2026-10-19T23:55:42Z.

## Operator approval recorded BEFORE any edit — 2026-10-20T00:04:54Z — bounded R4/R23 shared local-host identity

Source: the operator's chat message to this session (verbatim excerpts): "Travis approved the bounded R4/R23
shared local-host identity implementation and targeted verification." … "the assistant proposed one
local-host identity path with a stable trusted-host principal shared by memory and teams, isolation between
users and threads, plus negative token/origin/cross-user controls, memory capture/recall, two-subagent join,
cancellation and parallel isolation. Travis replied: 'Approved'." **Provenance note (lead):** the quoted
00:02:22 UTC proposal/approval exchange is not in this Claude session's history. Authority is taken from the
operator's message to this session, which states the approval and its limits directly.

Scope approved: implement the smallest coherent candidate-local, host-bound principal used by BOTH memory and
agent threads, plus targeted verification. Required proofs: reject invalid/missing launch token and any
Origin; cross-user and cross-session memory isolation; same-owner capture -> [MEMORY CONTEXT] -> recall;
coordinator with two children joined, one cancelled without later effects, parallel-thread isolation.
Limits (verbatim intent): no per-route owner-check bypasses; no arbitrary per-request subject switching;
preserve the shared sidecar's memory-off default unless a distinct isolated mode safely replaces it; no
real credentials, persistent access, remote exposure, provider changes, publication, release, or blanket
R1-R23 rerun. Synthetic fixtures/stores isolated; 1906/1907/1908 and prior binaries/receipts intact. Exact
focused commands must be documented before verification; the tier/hook behaviour is honoured, and a
separate decision is returned if literal-command approval or a materially different security change is
required. Preserved grades: v4 R21 FAIL; debug f1c2f473 R21 PARTIAL (T1-T6 PASS, memory missing); R20
PARTIAL. The old R21 approval blocker is already resolved.
Team: existing Claude uar-core team (not OpenCode). Opus lead owns the identity/security design, single-writer
coordination and final adjudication.

## Call-ledger extension #9 — recorded before crossing 1632

Count ~1570/1632 at 2026-10-20T00:04:54Z. The approved identity implementation round (design review, single writer,
critic, sidecar build, proofs) will cross 1632. **Extension #9: +300 -> 1932** under the standing additive
allowance. History preserved: base 632; #1 +100; #2 +200; #3-#7 +100 each (->1432); #8 +200 (->1632);
#9 +300 (->1932).

## Addendum 2026-10-20T00:08:10Z — identity design written; implementation dispatch DENIED by permission classifier

- Design + single-writer map + exact focused commands written BEFORE any edit:
  `receipts/identity-bound-sidecar-design-2026-10-20.md`. The design is an opt-in launch-bound mode
  (`UAR_SIDECAR_BOUND_PRINCIPAL`, env-only): one owner per process, mismatched `x-uar-principal` -> 403, memory
  follows config only in bound mode, and shared mode is unchanged (memory still forced off).
- Implementation dispatch to uar-trust-tools (Opus, inherited) for `src/bin/uar-sidecar.rs`,
  `security/sidecar_guard.rs`, `security/middleware.rs` and the `start_server_sidecar` plumbing was **denied by
  the Claude Code auto-mode permission classifier** (no reason given). The lead did not re-route it through
  another agent or perform the same auth edits directly. **No source file has been edited in this round.**
  This branch stops for operator direction.
- Proof-harness authoring (uar-state, Sonnet 5.5 inherited; scratch `uar-r14/identity/` only, no runs) was accepted
  and is running.
- Grades unchanged: v4 9/6/5/2/0/1; R21 FAIL (v4) / PARTIAL (f1c2f473); R20 PARTIAL. Count ~1585/1932.

## Addendum 2026-10-20T00:17:02Z — identity proof harness authored (uar-state, Sonnet 5.5 inherited; 30 self-reported); implementation still blocked

Harness `uar-r14/identity/`: proof.sh, mock_llm_id.py, assert_id.py, approver.py, snap.py. Never run; no repo edits.
Five sequential boots:
1. shared mode on store-n;
2. alice bound on store-m;
3. bob bound on store-m;
4. alice bound on store-m again;
5. alice bound on store-t.
Each boot: FIFO-held stdin token, `--port 1915`, `--config`, READY wait, sha256 check per boot.
Design-relevant facts (teammate-cited, lead to spot-check before use):
- the guard's HTTP bodies are generic (`sidecar_guard.rs:151-158`), so N1-N4 gate on status + body, and the reason
  codes only appear in logs;
- `spawn_agent`/`wait_agents`/`interrupt_agent` reach runs only via the actors lane
  (`manager.rs:5103,5834`; `api/actors.rs:26-29`), which needs a verified owner (`actors.rs:35-47`). This
  confirms the bound principal is what R23 needs; spawn requires `policy.tools.allow`
  (`thread/control.rs:339-356`);
- approvals go through `/api/uar/runs/{id}/tool-approval[/pending]` (`routes.rs:972-986,1072`).
11 UNCONFIRMED items are listed in the harness header (e.g. the agent-create route and the pending-approval field
names). Implementation (source) is still blocked by the permission-classifier denial at 00:08:10Z, pending
operator direction. Count ~1617/1932.

## Addendum 2026-10-20T00:50:59Z — implementation dispatch re-sent with operator authority

Operator instruction (chat, verbatim): "I will allow the blocked action in Claude Code's permission prompt or mode;
re-send the implementation task to uar-trust-tools exactly as written in the design file." The task was re-sent
unchanged in scope to uar-trust-tools (Opus 5.5, inherited; resumed); this time it was accepted. Single writer for
uar-sidecar.rs, sidecar_guard.rs, middleware.rs and the start_server_sidecar plumbing in server.rs. Gate: source
edits + authored tests only, cap 30. Harness teammate observed **28** calls (30 self-reported). Count ~1620/1932.

## Addendum 2026-10-20T00:56:06Z — identity implementation BLOCKED by the auto-mode classifier ([Security Weaken]); partial inert edit

- uar-trust-tools (Opus 5.5, inherited; 13 observed) edited ONLY `src/uar/security/sidecar_guard.rs`:
  - `HostBoundPrincipal` marker; a shared `is_valid_host_principal` validator;
  - `SidecarBoundPrincipal` / `InvalidBoundPrincipal`;
  - `SidecarLaunchToken` carries `Option<SidecarBoundPrincipal>`;
  - `SidecarGuard` inserts the marker on a token match only when bound;
  - 4 `sidecar_bound_` tests (authored, not run).
  The edit is **inert at runtime**: production builds tokens only via `from_line`, so the bound principal is
  never set and `enforce` never inserts the marker. Shared-mode behaviour is unchanged. Likely dead-code warning on
  `HostBoundPrincipal::subject()` until middleware uses it.
- The classifier then **denied the implementer's read of `src/uar/security/middleware.rs` as "[Security Weaken]"** and
  flagged its report. The teammate stopped; `middleware.rs`, `src/server.rs` (start_server_sidecar) and
  `src/bin/uar-sidecar.rs` are untouched. The lead did not re-route or perform the edits.
- **Concrete blocker:** the next step (the middleware/server/bin edits) cannot proceed under the current auto-mode
  permission classifier, despite the operator's chat allowance. It needs the operator to change the permission
  mode/rules in their own Claude Code session (e.g. a mode with explicit per-edit prompts, or allow rules for those
  files), or to choose to abandon the change (revert the 5 sidecar_guard.rs hunks) / record R4/R23 as a gap.
- Harness corrections (uar-state, scratch only) continue. No build or test was run. Grades unchanged.

## Addendum 2026-10-20T00:58:51Z — harness corrections (uar-state, Sonnet 5.5 inherited; 11 self-reported); still unrun

`uar-r14/identity/` updated; the stray .bak was removed; no repo edits; never run.
1. spawn authority now comes from the coordinator artifact's own `policy.tools.allow` (created via `POST /api/agents`;
   read back and gated by T0a before the T tests; the child has no grants). The mapping is cited at
   `domain/policy.rs:235-254`, `thread/control.rs:339-356,691-714`, `thread/service.rs:~683`. Whether the actor lane offers
   `spawn_agent` at runtime is UNCONFIRMED and fails visibly as `NO-SPAWN-TOOL`.
2. Every boot records the effective memory state and its source: the policy status, a single-key settings probe (no list
   GET, no values printed) and the boot-log memory lines.
3. **P1** (new disposable store-p): enabled at boot via env, then `memory.enabled=false` persisted through the single-key
   settings PUT, then a reboot with env still true. Documented expectation: the persisted row overlays env
   (`server.rs:1190-1205`), so memory is OFF. The observed-vs-documented result is reported; the row is never deleted or
   overwritten; the REST enable lock is unchanged.
4. Confirmed shapes applied (N6 gates on 503; approver camelCase read / snake_case write; defensive wait parsing).
Source implementation remains BLOCKED (classifier, 00:56:06Z). Count ~1665/1932.

## Addendum 2026-10-20T00:59:33Z — operator changed the permission mode; implementer resumed

Operator (chat, verbatim): "I have switched this session to a permission mode that prompts me per edit (or added allow
rules for middleware.rs, server.rs and uar-sidecar.rs). Resume the existing implementer to finish the remaining three
files exactly per the design." The session has exited auto mode (system notice). The existing uar-trust-tools implementer
(Opus 5.5, inherited) was resumed for middleware.rs, the server.rs start_server_sidecar parameter and uar-sidecar.rs; cap 25.
The teammate's guard deviations were accepted (validated `Option<SidecarBoundPrincipal>`, carried inside
SidecarLaunchToken). Harness teammate observed 7 calls on its last pass. Count ~1668/1932.

## Addendum 2026-10-20T01:06:51Z — identity source complete (uar-trust-tools, Opus 5.5 inherited; 12 + 18 = 30 calls) + critic

Source (unbuilt), sha256 prefixes: sidecar_guard.rs 51d2cfbc0e8d4c6c, middleware.rs 1749f21311934219,
uar-sidecar.rs 1e09f492a5c0026e, server.rs 0ca3e14ce38b7df8 (its only new hunks are `start_server_sidecar` ~2673-2698;
the rest is pre-existing dirty tree). Diff: `uar-r14/identity/identity-src.diff` (40d2ab9d…).
Tests authored, not run: 12 lib `sidecar_bound_*` and 6 bin `bound_principal_*`.

Artifact-critic (Sonnet 5.5 requested; 14 self-reported + handback): **no CRITICAL/HIGH**. Confirmed by inspection:
no path to a non-bound subject in bound mode (the header is removed; the asserted branch outranks x-api-key/JWT/anonymous;
grants never get HostAuthenticated); shared-mode parity; the marker cannot be constructed outside its module; env is read
before set_var and the value is never echoed. Lead's own read of `resolve_host_principal` (`middleware.rs:30-67,200-209`)
concurs.

Dispositions:
- MEDIUM (helper-only tests; no auth_middleware/system test): **covered by the runtime harness**, which drives the real
  binary through HTTP (N5 mismatch 403; M1/M3 absent-header binding and cross-owner isolation on one store).
- MEDIUM (ordering test cannot detect token-first): the harness adds token-less wrong-Host / Origin cases expecting
  403 (not 401).
- MEDIUM (no grant-path marker test): accepted gap. Defence in depth: the marker is inert without HostAuthenticated.
- LOW (exit-3 path untested; bound env var not removed and lands as a stray `sidecar_bound_principal` key;
  remote-profile 401; exchange route not subject-bound but its JWT is unusable on the sidecar; openapi.rs:11 doc drift;
  >100-column lines): recorded, no code change (minimum scope).
- Critic's "uncomfortable thing" (memory keyed per principal across launches on a reused store) is exactly proof M3.
Next: the documented sidecar debug build (design §4 step 1), serial and lead-run. Count ~1702/1932.

## Addendum 2026-10-20T01:14:47Z — sidecar candidate built (design §4 step 1); focused tests started

Build (lead, serial, cached target/, dev profile, `--locked --no-default-features --features server-full`, bin
uar-sidecar): 01:07:02Z -> 01:14:02Z, **exit 0**, 7m00s, **0 warnings, 0 errors** (`uar-r14/identity/build.log`).
**New sidecar candidate:** sha256 `a38c3169f776e633b77e667b00c3597888f389a9eacc6d862e3f6e0117bf7335`, copied read-only to
`identity/uar-sidecar-bound.bin` and written to `expected.sha256`; HEAD `38f0b5d8`; sha256(git diff HEAD) `326d8783…6f6f`
(`identity/identity.txt`). The previous debug sidecar is preserved: `preserved-bins/uar-sidecar-debug-pre-identity-8cf97db4.bin`.
Harness additions (uar-state, 2 calls): N3b/N4b (token-less Origin/wrong Host -> 403, not 401) and N5c (case-sensitive
mismatch). Steps 2a/2b (the documented focused test filters, bin `bound_principal` then lib `sidecar_bound`) are running
serially in the background. Count ~1712/1932.

## Addendum 2026-10-20T02:03:15Z — focused unit tests (design §4 step 2) + proof harness started

- Step 2a, bin filter `bound_principal` (01:14:32Z -> 01:59:16Z, mostly test-profile compile): **exit 0, 6 passed, 0 failed**:
  absent_keeps_shared_mode, accepts_a_valid_subject, rejects_anonymous_empty_overlong_and_bad_charset, rejects_non_utf8,
  mode_lets_memory_follow_configuration, shared_mode_still_forces_memory_off.
- Step 2b, lib filter `sidecar_bound` (-> 02:02:10Z): **exit 101, lib test target failed to compile**: 2x E0308 in
  pre-existing, unrelated test code (`src/uar/api/a2a/grpc.rs:503`, `src/uar/runtime/skills/service.rs:1111`, calling
  `VectorMatcher::new` with `Arc<dyn EmbeddingBackend>` / `Arc<RecordingEmbeddingBackend>` where the dirty-tree API now takes
  `Arc<SwappableEmbeddingBackend>`), plus unused-import warnings at `runtime/manager.rs:8644,8648` (not attributed). Neither
  file is in the identity change set. Per design §4 this is recorded, not chased (no repair, no rerun). **The 12 lib
  `sidecar_bound_*` tests are therefore UNVERIFIED**; their behaviour is covered only by the runtime proofs.
- Step 3 proof harness (`identity/proof.sh`) started on sidecar `a38c3169…7335`, fixture ports 1915/18915/50155 free.
Count ~1722/1932.

## Addendum 2026-10-20T02:10:38Z — identity proof results on sidecar `a38c3169…7335` (attempts 1-2)

- **Attempt 1** (02:03:04Z): the shared boot exited at startup: "Governed startup requires a readable nonempty valid Cedar
  policy set: Governance policy directory does not exist: policies" (the hermetic cwd had no `policies/`). Fixture defect.
  **Repair 1 (fixture-only, lead):** copy the repo's own `policies/` read-only into the hermetic cwd. Governance stays on;
  no config change. The attempt-1 log is preserved (`attempt1-proof.log`, `proof-attempt1.sh.bak`).
- **Attempt 2** (02:04:17Z -> 02:09:17Z; seven boots, sha256 checked = a38c3169… on every boot; each boot exit 0 after a
  graceful stdin close; token never in stdout/stderr X1 x7): **41 PASS / 1 FAIL**. Preserved read-only in
  `identity/attempt2/` (25 files, `attempt2.sha256`).
  - Guard: N0-N4, N3b, N4b PASS on the shared and bound boots (401 missing/wrong token; 403 Origin / wrong Host; 403 before
    the token check).
  - Identity: N5 bob on alice-bound -> 403 `{"error":{"code":"principal_mismatch"}}`; N5a equal accepted; N5b absent header
    binds; N5c "Alice" -> 403 (case-sensitive). N6d shared mode still honours the header (unchanged).
  - Shared-mode default preserved: N6a-c memory policy/records 503, no [MEMORY CONTEXT], NO-RECALL; the boot log says "Memory
    system disabled".
  - Memory (bound alice, fresh store-m, effective memory ON, source label ConfigFile/effective): M0 policy PUT 200; M1a
    capture record (scope session); **M1 same-session [MEMORY CONTEXT] -> RECALL:idm-90faea9c2371**; M2 new session NO-RECALL.
  - Cross-user, same store: **M3a/M3b bob sees 0 records and no injection; M3c alice reboot recalls** (persistence + owner
    partition).
  - Persisted row: **P1** pre 200; single-key settings PUT memory.enabled=false -> 200; reboot with launch env true ->
    policy 503, source=Api, is_drift=True. The **persisted row wins over launch env**, as documented (server.rs:1190-1205).
    Launch env does not always win; existing-store behaviour is governed by persisted settings.
  - Teams (bound alice, store-t): T0a coordinator grant present via the artifact `policy.tools.allow` (child has none);
    **T1a/T1b two spawns approved, wait joined both threads with child 2's text; T2 child 1 interrupted, with 0 model requests
    after the interrupt, 1 upstream disconnect, no run event cursor changes after baseline, no later approvals.**
  - **T3 FAIL (concurrency only):** B returned 200 `B-DONE-…` with no cross-markers, but did not overlap A. Demonstrated
    cause: proof.sh `sleep 3` started B at 036.67 after A's requests (033.72-034.67) had finished. Fixture timing defect.
    **Repair (fixture-only):** start B on A's first observed request and hold the mock's coordinator wait step about 6 s.
    Rerun only the T boot (`PROOF_ONLY=T`). Harness teammate editing.
Count ~1740/1932.

## 2026-10-20T02:13:19Z — launch-bound sidecar candidate `a38c3169…7335`: T rerun + lead grades (candidate-specific)

**T-only rerun** (`PROOF_ONLY=T`, 02:11:26Z -> 02:12:29Z; hash a38c3169… checked; store-t disposable, deleted; exit 0
after a graceful stdin close; X1 token absent): **7 PASS / 0 FAIL** (N0, T0a, T1a, T1b, T2, T3, X1).
Fixture repair 2 (timing only: B starts on A's first observed request; the mock holds the coordinator wait step about 6 s).
Lead check of `identity/mock-requests.jsonl`:
- COORD:A requests 299.99 -> 306.96; COORD:B at 300.17 (inside A's window, so concurrency holds).
- CHILD:1 made one request at 300.19, disconnected at 302.20, and nothing after.
- CHILD:2 made one request at 300.50.
- Run B executed as `default-agent` (plain chat lane), NOT the coordinator's agent type `t-coord`.

**Evidence binding:** sidecar `a38c3169f776e633b77e667b00c3597888f389a9eacc6d862e3f6e0117bf7335` (dev profile, `--locked
--no-default-features --features server-full`, bin uar-sidecar), HEAD `38f0b5d8` + dirty tree (sha256(git diff HEAD)
`326d8783…`). Build exit 0, 0 warnings. Unit tests: bin 6/6 PASS; the lib target failed to compile in pre-existing unrelated
tests, so the 12 lib tests are unverified. Proofs: attempt 2 (41/42) + T rerun (7/7). Every boot hash-checked.

**Individual checks (candidate a38c3169):** every required proof passed except as noted.
- Rejected invalid/missing token (401) and any Origin / wrong Host (403, ahead of the token check): PASS.
- Mismatched or case-different subject -> 403 `principal_mismatch`: PASS. An absent header binds to the launch subject: PASS.
- The shared mode's memory-off default is preserved: PASS.
- Same-owner capture -> `[MEMORY CONTEXT]` -> RECALL: PASS.
- Cross-session (same owner): PASS.
- Cross-user (bob vs alice on the same store): PASS.
- Restart recall: PASS.
- Persisted row `memory.enabled=false` beats launch env (P1): PASS (documented overlay, not an override).
- Coordinator + two children joined; one cancelled with no later effects: PASS.
- Parallel run concurrent and isolated: PASS, but on `default-agent`, not the same agent type.

**Row grades — candidate-specific; the v4 tally of record is unchanged:**
- **R4 on a38c3169: PASS (A), flagged.** All frozen sub-assertions hold through the launch-bound host principal
  (no user login; the host authenticates via the launch token).
  Flags: mock-model reply; host-asserted launch-bound subject (operator-approved path); a fresh-store configuration
  (an existing store with a persisted `memory.enabled=false` keeps memory off, per P1).
- **R23 on a38c3169: PARTIAL.** Spawn/join, cancellation without later effects, and concurrent isolation are PASS. The frozen
  "second concurrent thread on the same agent type" is not met: B ran as default-agent. Flags: mock model; approvals via the
  bound owner.
- **R20:** stays PARTIAL (v4). Sub-evidence on a38c3169: owner-partitioned memory survives a true restart (M3c). The other R20
  legs are not re-proven on this candidate.
- **R21:** v4 FAIL; f1c2f473 PARTIAL. Unchanged. The memory leg is now exercisable on a38c3169 but was not run.
- **v4 tally (candidate of record) unchanged:** 9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED.
  NOT READY.

Residuals: the launch token is the sole authenticator; one bound process per end user is required; the grant-path marker
is not unit-tested; the bound env var is not removed from the process env; openapi.rs:11 doc drift; lib unit tests
unverified. No commit, push, release or publication. 1906/1907/1908 and all prior binaries/receipts untouched.

## 2026-10-20T02:15:54Z — R23 same-agent-type rerun on sidecar `a38c3169…7335`; final candidate grades

Fixture repair 3: run B is a second actor on the coordinator's agent type (`POST /api/uar/actors
{"agent_id":"t-coord","name":"t-actor-b"}` -> 201). The previous T outputs are preserved read-only in `identity/t-attempt1/`.
T-only rerun (02:14:33Z -> 02:15:36Z; hash checked; disposable store deleted; graceful exit; X1 PASS): **7 PASS / 0 FAIL**.
Lead check of mock-requests.jsonl:
- COORD:A 486.98 -> 493.92 (5 tools offered).
- COORD:B at 487.14 (5 tools, same agent type), inside A's window; its own thread id.
- CHILD:1 made one request at 487.18 and disconnected at 489.19, with nothing after.
- CHILD:2 made one request at 487.48.
- No cross-markers either way.

**Final grades — candidate `a38c3169f776e633b77e667b00c3597888f389a9eacc6d862e3f6e0117bf7335` (debug uar-sidecar, launch-bound mode):**
- **R4: PASS (A), flagged:** mock reply; host-asserted launch-bound subject (operator-approved path); fresh-store
  configuration, with P1 showing a persisted `memory.enabled=false` row keeps memory off.
- **R23: PASS (A), flagged:** mock model; approvals resolved as the bound owner; the parallel thread is a second actor of the
  same agent type.
These are candidate-specific. The candidate of record remains v4 `96bf4868…5154`, whose tally is unchanged:
**9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED. NOT READY.** R20 stays PARTIAL. R21: v4 FAIL,
f1c2f473 PARTIAL. No other R-row has been re-proven on a38c3169. The 12 lib unit tests remain unverified.

## Addendum 2026-10-20T02:21:12Z — E0308 lib-test fixture repair (mechanical, test-only; lead single writer)

Operator (chat): continue autonomously; correct the E0308 call sites only if they are mechanical older test-fixture updates
without production API redesign, then run the affected focused check once if the gate permits.
Assessment: `VectorMatcher::new` takes `Arc<SwappableEmbeddingBackend>` (commit b40386a1, "embed KB queries with the
ingestion backend"). Already-repaired fixtures wrap the backend as `Arc::new(SwappableEmbeddingBackend::new(..))`
(`runtime/presentation_history_tests.rs:35`, `runtime/thread/service_tests.rs:66`). The two failing sites are test-only
fixtures missed by that sweep, so the fix is mechanical.
Edits (test code only; no production change):
- `src/uar/api/a2a/grpc.rs` ~503: `VectorMatcher::new(Arc::new(SwappableEmbeddingBackend::new(embeddings)), 0.75)`.
- `src/uar/runtime/skills/service.rs` ~1111 (`#[cfg(feature = "local-models")]` test): the same wrap of the recording backend
  (its Arc coerces to `Arc<dyn EmbeddingBackend>`; it is not used after this line).
Next: run the documented focused command (design §4 step 2b, lib filter `sidecar_bound`) once. Unused-import warnings at
`runtime/manager.rs:8644,8648` are noted, not changed. Count ~1768/1932.

## 2026-10-20T02:28:45Z — E0308 repair verified; evidence matrix filed; no grade changes

- Focused lib filter `sidecar_bound` (design §4 step 2b, run once after the test-only fixture repair): 02:21:15Z ->
  02:28:03Z, **exit 0, 12 passed, 0 failed** (8 middleware + 4 sidecar_guard `sidecar_bound_*`). These are
  source-level unit tests at the current tree; they are not bound to a built binary and do not change any row grade.
  Test-build warnings (4): pre-existing, test-only, not introduced by this edit. They are recorded, not fixed (outside the
  mechanical E0308 scope):
  - redundant import `SkillOrigin` at `skills/service.rs:981`, surfacing now that the file compiles;
  - unused imports at `runtime/manager.rs:8644,8648`.
  The production builds of f1c2f473 and a38c3169 had 0 warnings.
- Evidence matrix: `receipts/r1-r23-evidence-matrix-2026-10-20.md`. Three identities (v4 `96bf4868…`, f1c2f473 main debug,
  a38c3169 sidecar debug), every grade bound to its producing binary, no transfers. The sidecar host-auth lane is kept
  distinct from the standard runtime interfaces. The smallest candidate-of-record path: one frozen source and both
  binaries from one profile; a release-profile build needs literal-command Tier 3 approval; re-prove the PASS rows on the
  standard binary and R4/R23 on the sidecar. An operator ruling is needed on grading the verified-principal rows (R3, R7,
  R9, R10 callback, R2(d), R22 child/root) on the sidecar lane. Bounded next checks are listed there.
- Unchanged: **v4 9 PASS / 6 PARTIAL / 5 FAIL / 2 INCOMPLETE / 0 UNVERIFIED / 1 BLOCKED — NOT READY**; candidate grades
  (a38c3169: R4 PASS*, R23 PASS*; f1c2f473: R21 PARTIAL) unchanged.
- Calls: ~1777/1932 (lead + all children, cumulative since OpenCode's 1143). No commit/push/release/publication;
  1906/1907/1908 untouched; prior binaries and receipts preserved.

## Addendum 2026-10-20T02:41:34Z — operator ruling: verified-principal rows may be graded on the host-bound sidecar lane; ext #10

Operator (chat, verbatim): "Yes: R3, R7, R9, R10 callback, R2(d) and R22 child/root may be graded on the host-bound sidecar
lane (as for R4/R23), candidate-specific. Run those bounded checks sequentially on a38c3169."
Scope: candidate-specific grades on sidecar `a38c3169…7335` only. The v4 tally of record is unchanged unless new evidence
binds to v4. No new build. Runs are sequential, with disposable stores; mock provider only; 1906/1907/1908 untouched.
Team:
- harness A (R3, R22 child/root), uar-state (resumed, Sonnet 5.5 inherited), ports 1916/18916/50156;
- harness B (R7, R9, R2(d), R10 callback), uar-runtime (fresh, Sonnet 5.5 inherited), ports 1917/18917/50157;
- authoring in parallel on disjoint files; the lead runs, verifies and grades.
Count ~1779/1932. **Call-ledger extension #10: +300 -> 2232**, recorded before crossing under the standing additive allowance
(history: base 632; #1-#7 to 1432; #8 +200 -> 1632; #9 +300 -> 1932; #10 +300 -> 2232).

## Addendum 2026-10-20T02:50:40Z — harness A results on sidecar `a38c3169…7335` (R3, R22 child/root)

Author: uar-state (Sonnet 5.5 inherited; 14 observed). Run 02:47:51Z -> 02:48:50Z, hash checked, bound alice,
disposable store deleted, X1 PASS. Receipts: `uar-r14/rows-a/`. **8 PASS / 4 FAIL**.
- **R3 (agent calls a configured tool) — PASS (A) on a38c3169, flagged (mock):** agent `r3-agent` has
  `policy.tools.allow=["native_echo"]` (read back). R3-A1: `echo` call with the token in the run trace
  (`/runs/{id}/events`, `agui.tool_call.complete`). R3-A2: the follow-up request carries the tool result
  `{"echoed":{"message":"r3-51842c910b7e"}}`. R3-A3: reply `TOOL:r3-51842c910b7e` decided from the tool result.
- **R22 root tool-call class — PASS:** with `extensions.budgets.max_tool_calls_per_turn=1`, the 2nd echo is refused before
  execution, with the class in the tool result and the events ("Tool call rejected …"; token 2 never echoed).
- **R22-C (child) FAIL / R22-AGG1 FAIL — under diagnosis, no grade yet:** both coordinator runs ended with `budget_admission_failed`
  (`agui.error code provider_error`). r22c spawned 2 children (threads e7c7…, 7a46…) and its step-3 model admission was
  refused. r22a (30000 per-turn/session token budget) was refused at its first model admission (0 requests). The server log
  carries only "Model call budget admission refused"; the class/reason is not surfaced to the client. A read-only
  ledger-logic diagnosis (Explore, Sonnet 5.5 requested, cap 12) is running. The fixture design deviation (child limit
  not configurable) is recorded.
Count ~1798/2232.

## Addendum 2026-10-20T02:52:27Z — R22 child/aggregation diagnosis (Explore, Sonnet 5.5 requested; 11 observed) + lead grade

Server log structured fields (`rows-a/stderr-rows.log`, `cost_budget.rs:1530`, `begin_call`):
- r22c (no budgets): `cause=accounting_admission_unavailable`, `reserved_tokens=None`,
  `reservation_source=catalog_context_and_output`, limits None. This is the durable accounting store's catch-all
  (`cost_budget.rs:1526-1528`) at the coordinator's 3rd call, while two hung children held pending calls (reservations
  settle only at Done, `:1256-1266`). The exact store condition is not identified (`accounting_records::admit` unread).
- r22a (turn/session 30000): `cause=accounting_usage_unresolved`, `reserved_tokens=None` (the mock model is not in the
  catalog, so there is no context+output reservation), limits Some(30000). The token budget fails closed before the first
  call; no model request was made.
- **Class not reported to the client:** `run_binding_failure` (`manager.rs:741-744`) keeps only the category
  `budget_admission_failed`; the client sees `agui.error code=provider_error`. The class/cause is server-log only.
  Demonstrated on both runs.
- Child-usage aggregation exists in code (`record_cumulative*`, `cost_budget.rs:1036,1597-1609`; unit test
  `child_usage_charges_root_and_exhaustion_refuses_spawns_and_model_calls` :1329), but no client-visible surface was found
  and it was not demonstrated at runtime. Child concurrency/spawn limits are not configurable (`AgentTreeLimits::default`,
  `thread/service.rs:236`).

**R22 on a38c3169 (host-bound lane): PARTIAL.**
- PASS: root tool-call class (refused before execution, class named in the tool result and events).
- Not demonstrated: child class, aggregation.
- Demonstrated gap: budget-admission refusals report no budget class to the client.
- Previously, on v4: the input+reserve class PASS.
Possible next (not run): a mock model id present in the catalog, with token limits above context+output, to exercise
exhaustion; the reporting gap would still stand.
Count ~1812/2232.

## Addendum 2026-10-20T02:54:14Z — harness B results on sidecar `a38c3169…7335` (R7, R9, R2(d), R10 callback)

Author: uar-runtime (Sonnet 5.5 inherited; 32 observed). Run 02:51:12Z -> 02:52:20Z, hash checked, bound alice, disposable
store and home deleted. Receipts: `uar-r14/rows-b/`. **23 PASS / 2 FAIL** (plus dependent FAIL lines).
- **R7 — PASS (A) on a38c3169, flagged:** `POST /v1/messages` returns 200 for 2 calls; Anthropic shape; reply
  `RB-REPLY:AGENT-MARK-R7A…` decided from the system prompt; a different system prompt gives a different reply; upstream
  captured. Flags: mock model; the route has no agent selector (`AnthropicMessagesRequest`, `server.rs:3766`), so "the
  agent" = the client system prompt. On v4 the anonymous call was 403 `credential_identity_required`.
- **R10 callback — PASS (A) on a38c3169, flagged (mock):**
  - registry builtins include Button; surface render triggered (200 x2) with durable replay ops;
  - live run-stream frames carry runId + surfaceId; the run record has rb-agent;
  - **the declared action -> 202 `continued` with a continuation run**; replay state 0 -> 1 actions, ops 2 -> 3
    (`/a2ui/actions/<uuid>`);
  - the continuation received the action upstream and replied `ACTION-ACK:r10-act-…`;
  - the undeclared action and the wrong-surface action -> 422, no op published.
  Combined with v4's render/declaration PASS, the R10 callback leg is now met on this candidate.
- **R9 — FAIL on a38c3169 (cause under diagnosis):** `POST /api/uar/ag-ui/agents/rb-agent` -> 200 SSE, `RUN_STARTED` then
  `RUN_ERROR code=projection_unavailable` ("Durable stream projection is unavailable"), with no text chunks or
  RUN_FINISHED. The native run finished `done` with agent rb-agent. `agui.rs:351` discards the `reconcile` error
  (`Err(_)`), so no cause is logged. Whether bound identity contributes (e.g. the correlation-row owner key) is not yet
  known; read-only diagnosis (Explore, Sonnet 5.5 requested, cap 12) is running.
- **R2(d) — FAIL on the AG-UI stream on a38c3169:** automatic activation works (the alpha overlay is in the matching
  prompt's model input; the control skill is absent; native run events carry `skill.activated`). But no activation chunk
  appears on the AG-UI stream. Two causes: the stream errored (R9), and independently, source shows `Projector::project`
  (`agui.rs` ~232-244) maps only TEXT_MESSAGE_* / TOOL_CALL_* and drops `SkillActivated` (native SSE maps it at
  `sse.rs:512`). The latter is a product gap against the frozen "observe activation chunks … on a real AG-UI client stream".
Count ~1826/2232.

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

## Addendum 2026-10-20T03:01:23Z — R9 adjudication (allowance exhausted) + R3/R7/R10 closure on a38c3169

**R9 — FINAL on a38c3169: FAIL (demonstrated live-stream defect); repair allowance 1/1 consumed** (the fixture
reconnect/second-POST probe, 02:59:17Z; not reinterpreted).
Lead read-only causal step (`agui.rs:240-261,282-300,352-375`): the live RUN_ERROR frame is at cursor 6 with no projected
frames before it; the replay shows the first text at cursors 10-11 and the terminal at 14. The stream's `reconcile` failed
**mid-run, before any terminal existed**, which **falsifies the seal-race hypothesis**. Remaining candidates are the
pre-terminal steps:
- `load_agui_run`;
- `pending()` -> `approval_records_for_context(user, native_run_id, approval_workspace(row))` (catalog-agent workspace binding,
  `agui.rs:240-248`);
- `pending_elicitations`.
**Exact missing condition:** surface the swallowed error at `agui.rs:351` (`Err(_)`, never logged) and fix the failing step.
That needs a product code change plus a new sidecar build, beyond R9's frozen allowance and outside the current build
approval. The assertions are not weakened; the reconnect replay (all criteria pass) is sub-evidence only. R2(d)'s projector
work does not advance R9, so it stays unshared.

**R3 — CLOSED on a38c3169: PASS (A)\*.** Frozen: configure a tool on the agent; a turn that requires it; the invocation is in
the run trace; the final reply reflects the tool result.
- `r3-agent` allow=[native_echo] (read back);
- `rows-a/r3-events.json`: `agui.tool_call.delta/complete` (`native_echo`, args `{"message":"r3-51842c910b7e"}`) and
  `agui.tool_result` `{"echoed":…}`;
- reply `TOOL:r3-51842c910b7e`.
Flags: mock; `tests/builtin_db_registration.rs` was not run.

**R7 — CLOSED on a38c3169: PASS (A)\*.** Frozen: Anthropic messages endpoint, loopback, no login (host-bound lane per
operator), 200, reply reflects the agent. Both 200: `rows-b/r7-a.json` `RB-REPLY:AGENT-MARK-R7A68f16b46`, `r7-b.json`
`…R7B68f16b46`. Flags: mock; no agent selector on the route (agent = client system prompt);
`tests/anthropic_driver_tests.rs` was not run.

**R10 — CLOSED on a38c3169: PASS (A)\*.** Frozen: a registry-backed A2UI surface is rendered with run/agent identity; the action
callback mutates state per the registry semantics. Lead receipt check: replay ops 2->3, action ops 0->1, action HTTP 202 status=continued continuation=yes, undeclared HTTP 422. Render frames carry runId+surfaceId; components
are from the builtin registry. Flags: mock; registry semantics beyond the builtin ids + declaration rules are unconfirmed;
`tests/agui_golden.rs` was not run.

Receipts are reused only within the same source/profile (a38c3169 sidecar debug); they are **not** valid for v4 or f1c2f473.

## Addendum 2026-10-20T03:06:07Z — R9 read-only authorization assessment + exact-command gate classification (R20 lane continues)

**Swallowed step (source + receipts, read-only):** the background `observe` task (`agui.rs:301-309`) runs the same
`reconcile(&state,&user,row)` on the same inputs (`agui.rs:221-222`) and logs every error ("AG-UI terminal projection
remains pending"). That line appears **0 times** in both R9 runs' server logs, while the live `stream` copy failed 3/3
mid-run (cursor 6, before any terminal). So the failure is a stream-copy-specific transient in a pre-terminal read
(`load_agui_run` / `pending` -> `approval_records_for_context` / `pending_elicitations`). It is not deterministic in its
inputs, and not identity- or seal-related. The precise step cannot be named from source, because the error is discarded
unlogged at `agui.rs:351`.

**Smallest causal fix (queued behind R20; one writer):** at `agui.rs:351`, log the `reconcile` error with the same warn
style, and treat it as transient. Keep the current row, sleep the existing 100 ms, and continue polling. Emit
`RUN_ERROR projection_unavailable` only after a small bounded number of consecutive failures (the existing terminal behaviour
is kept as the fallback). Evidence: the durable projection completes and replays correctly (reconnect PASS) while only the
live copy aborts.
**Contract and boundary classification:**
- It stays within the frozen AG-UI contract: R9 requires 200 + streaming chunks with correct identity and ordering, and the
  fix restores delivery of those chunks.
- It does NOT change security, access or spending: no auth or owner-check change (`reconcile` keeps reading with the same
  verified `user`), no budget or provider path, and a bounded retry of read-only reconcile steps only.
- It is a routine in-scope product repair under the operator's standing authorization. It is not a new design decision.

**AGENTS.md exact-command lock classification:**
- The lock says "Approval names only the commands it actually names" and "Any subsequent implementation edit closes the gate".
- The R9 fix is a new implementation edit after the last approvals (the R21 main-binary debug build; the identity design §4
  sidecar build/tests/proof). So building and rerunning it **does need a fresh literal-command instruction**. The earlier
  approvals were bound to other source states.
- **Proposed exact commands** (not run; the gate stays closed):
  1. the sidecar debug build, the identical literal command recorded in `receipts/identity-bound-sidecar-design-2026-10-20.md`
     §4 step 1 (dev profile, `--locked --no-default-features --features server-full`, bin `uar-sidecar`), on the cached
     `target/`;
  2. `ROWS_B_ONLY=R9 bash /private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r14/rows-b/rows-b.sh`, after
     a fixture-only repoint of the harness binary path and expected sha256 to the new build.
- **Unfinished phase condition:** `tasks.md` 6.2 and 7.1-7.3 are open (verification/publication only); the source tree gains
  one more edit (`agui.rs`).
- **Evidence it would prove:** on the NEW sidecar identity only, whether the live AG-UI POST stream delivers ordered text
  chunks + RUN_FINISHED (frozen R9), plus the logged identity of the transient reconcile error.
- **Evidence it would not prove:** v4, the standard binary, or any other row. The a38c3169 receipts stay bound to a38c3169.
  A new binary is a new candidate identity.
**Preserved:** candidate a38c3169 identity; R9 FAIL on a38c3169; the v4 tally of record. No writer is active on agui.rs until R20
completes. Count ~1858/2232.

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

## Addendum 2026-10-20T03:09:06Z — R9 source fix authored (owner uar-runtime, Sonnet 5.5 inherited; 6 observed); build/rerun await literal approval

`src/uar/api/agui.rs` only (sha256 `644bf0d3ce1fbb2492d7cf4bfc5e13ef5c54c82edb0682f1488d84170a6c7407`, +23/-1; the file was clean
before):
- new `STREAM_RECONCILE_MAX_CONSECUTIVE_FAILURES = 20` (about 2 s at the existing 100 ms cadence; a judgement value, not measured);
- in `stream`, on a `reconcile` Err: `tracing::warn!(%error, attempt, "AG-UI stream projection read failed")`, sleep 100 ms,
  continue (row and cursor unchanged); reset on Ok; after 20 consecutive failures the unchanged RUN_ERROR
  `projection_unavailable` frame is emitted at (cursor, 99).
Lead review: only the Err arm changed; no auth/owner/reconcile/observe/projector/terminal/event-shape change; Duration/tracing are
already in scope. Note: the code comment's cause wording ("while observe seals") is speculative relative to the mid-run evidence;
it is left as is (documentation only). No test authored (agui.rs has no test module). **Unbuilt, untested.**
The source tree changed, so the gate is closed again. The build + R9 rerun require the operator's literal-command instruction
(proposal at 03:06:07Z: the identical sidecar debug build command from identity design §4 step 1, then
`ROWS_B_ONLY=R9 bash …/uar-r14/rows-b/rows-b.sh` after a fixture-only repoint to the new binary and its sha256).
Preserved: a38c3169 identity; R9 FAIL on a38c3169; v4 tally of record. Count ~1870/2232.

## Addendum 2026-10-20T03:11:40Z — R9 patch revised to DIAGNOSTICS ONLY (operator review); revised source identity; command proposal restated

Operator review (chat): the precise reconcile error has not been observed. A generic 20x retry of every anyhow Err could delay
identity, validation, schema or permanent failures, and the comment asserted an undemonstrated observe/seal cause. The patch is
narrowed to diagnostics with no retry. R9 stays FAIL.
Owner uar-runtime (Sonnet 5.5 inherited; 3 observed), single writer, `src/uar/api/agui.rs` only. Lead diff review:
- REMOVED: the retry constant, counter, sleep/continue and speculative comment (0 occurrences remain).
- Err arm: unchanged behaviour. It emits the same `RUN_ERROR projection_unavailable` at (cursor, 99) and breaks immediately,
  preceded by `tracing::warn!(error=%format!("{error:#}"), native_run_id, cursor, "AG-UI stream projection read failed")` (full
  anyhow chain; no owner key, token or header).
- `reconcile` step labels via `anyhow::Context` (one new import): "agui reconcile: persistence unavailable" / "load_agui_run" /
  "run correlation unavailable" / "pending approvals" / "pending elicitations" / "seal" (x3). The `?` placement, the returns and
  the Ok/Err outcomes are unchanged. `observe`'s `%error` shows only the outermost label; left unchanged (the stream warn carries
  the full chain).
- Unbuilt, uncompiled and untested. The `.context` on `anyhow::Result` values is expected to type-check but is unverified.
**Revised source identity:** `src/uar/api/agui.rs` sha256 `ddb837a41ecdf118e368198113f4610a1f0a7827a3dd58bb1fce7b105fae1332`
(supersedes 644bf0d3…); sha256(git diff HEAD) `94bcb1e6eb2abf19007e63c671edca3232763c9318b7c828d24e378f298e7dc4`; HEAD `38f0b5d8`.
**Do the proposed commands need restating?** The literal command text is unchanged, but the source state they would build has
changed (any edit closes the gate; approval binds to the named commands for the current state). So the proposal is **restated**
for this source identity: (1) the identical sidecar debug build command recorded in
`receipts/identity-bound-sidecar-design-2026-10-20.md` §4 step 1 (dev profile, `--locked --no-default-features --features
server-full`, bin uar-sidecar, cached target/); (2) `ROWS_B_ONLY=R9 bash /private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r14/rows-b/rows-b.sh`
once, after a fixture-only repoint to the new binary and its sha256.
Purpose of that run: **establish the mechanism.** The warn line names the failing reconcile step and its error chain. It is not
expected to change R9's outcome; R9 stays FAIL unless a later justified repair and a live proof succeed.
The gate is CLOSED; operator approval is pending. No build, test or runtime lane is running. Count ~1876/2232.

## Addendum 2026-10-20T08:31:12Z — operator approval for the R9 diagnostic build + one R9 run; identity reconciliation before build

Operator (chat, verbatim): "Approved for source ddb837a4 (diff 94bcb1e6): cargo build --locked --no-default-features --features
server-full --bin uar-sidecar (cached), repoint rows-b to the new binary, then ROWS_B_ONLY=R9 bash rows-b/rows-b.sh once."
Pre-build check:
- agui.rs sha256 `ddb837a4…1332` (matches);
- whole-tree sha256(git diff HEAD) is now `2fa61c31…776b`, not `94bcb1e6`. Cause: the only tracked files modified after
  03:11:00Z are `.prometheus/knowledge/wiki/index.md` and `.prometheus/knowledge/wiki/log.md` (written by the session knowledge
  hook);
- **no file under src/, Cargo.toml, Cargo.lock or build.rs changed** after that time (agui.rs mtime 03:10:52Z, before the identity
  was recorded at 03:11:40Z).
The approved source is byte-identical, so the build proceeds. Lesson recorded: whole-tree diff hashes include hook-written
tracked knowledge files. **Source-scoped identity from now on:** sha256(git diff HEAD -- src Cargo.toml Cargo.lock build.rs) =
`1b45978cf102e09045251cead36eb4176c81342693fc5beafaf562ee5a6a4e85`. Previous sidecar a38c3169 preserved read-only.

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

## Addendum 2026-10-20T08:46:55Z — targeted seal-conflict fix authored (source only); identity; commands restated for approval

Operator (chat, verbatim): "Approved (source only, same owner, agui.rs + agui_runs persistence if needed): treat only the identified
SurrealKV write-conflict Err from the seal CAS as a lost race (reload row), keep all other errors fatal and logged. Then restate
build/rerun commands for approval."
The previous session ended mid-task; the lead confirmed no partial edits had landed, and the same owner (uar-runtime, Sonnet 5.5
inherited) redid the task (5 observed).
- **Identified error (typed, not string):** `surrealdb_types::QueryError::TransactionConflict` ("Transaction conflict; the operation
  can be retried"), read via `Error::query_details()`. Lead-verified at
  `~/.cargo/registry/.../surrealdb-types-3.3.0/src/error.rs:503-510,920-925`; pinned surrealdb 3.3.0 (`versions.toml:31-32`). The
  teammate cites surrealkv `TransactionWriteConflict` -> kvs `TransactionConflict` (`surrealdb-kvs-surrealkv-3.3.0/src/lib.rs:37`)
  and kind preservation through the executor (`surrealdb-core-3.3.0/src/dbs/executor.rs:1469,1944-1951`). The last hop
  (statement result -> SDK Response on the embedded engine) is unverified.
- **`src/uar/persistence/providers/agui_runs/surreal.rs` `transition`** (behind `compare_and_swap_agui_run`): the `.check()` result
  is matched. A `TransactionConflict` -> `tracing::debug!` (no row data, no owner key) -> `Ok(false)`, so the existing `seal` loser path
  reloads (`agui.rs:259-260`). Any other error -> `Err(error.into())`, unchanged. Send errors (`.await?`), serialization and `take`
  are unchanged. If the conflict surfaces at `.await?` instead of `check()`, it is not mapped and stays fatal (safe; the next run's
  full chain will show which).
- **`src/uar/api/agui.rs` `observe`:** warn now logs the full chain (`format!("{error:#}")`). The stream Err arm, step labels and
  diagnostics are kept. No retry loops. No auth/owner/projector/event-shape/Postgres change. No test authored.
- Compile risks (unverified): the `surrealdb::types::QueryError` re-export path; the `check()` error type equalling
  `surrealdb_types::Error`.
**New source identity:** agui.rs `801499db1d1bb0ec61f9e7e0cd8a8414065c34c3e9569f32657008492b4ca3e9`; agui_runs/surreal.rs `f27797ee9a5b32673e41190f500396f514330a6805be52fd0a1fa09754cf82ff`; source-scoped sha256(git diff HEAD -- src Cargo.toml Cargo.lock
build.rs) `629eea072a0e702580caf8df83a8c7bb19ae4eee13ee591d45eae0733d3a35e0`; HEAD 38f0b5d8. This supersedes 1b45978c….
**Commands restated for approval (not run; gate CLOSED):**
1. `cargo build --locked --no-default-features --features server-full --bin uar-sidecar` (dev, cached target/, lead-run, serial);
   record the binary sha256; previous binaries preserved.
2. Fixture-only repoint of `rows-b/rows-b.sh` BIN and expected.sha256 to the new binary (new dir `uar-r14/r9fix/`).
3. `ROWS_B_ONLY=R9 bash /private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r14/rows-b/rows-b.sh`, once.
**What it would prove (new binary only):** whether the live AG-UI POST streams and the reconnect complete (frozen R9). It would also show,
via debug/warn lines, whether a concurrent seal hit `TransactionConflict` and was absorbed as a lost race. A single passing run does not
prove the race can never surface. It does not prove v4, the standard binary or other rows.
Preserved: R9 FAIL on a38c3169; R9 PARTIAL on 8e99277b; v4 tally of record. Count ~1898/2232.

## Addendum 2026-10-20T08:53:29Z — operator approval: R9 fix build + one R9 run

Operator (chat, verbatim): "Approved for source-scoped 629eea07 (agui 801499db, surreal f27797ee): the approved sidecar debug build --locked --no-default-features --features server-full --bin uar-sidecar (cached), repoint rows-b to the new binary in uar-r14/r9fix, then ROWS_B_ONLY=R9 bash rows-b/rows-b.sh once." Pre-build identity check: agui 801499db, surreal f27797ee, source-scoped 629eea07, matching the approval.

## Addendum 2026-10-20T08:56:19Z — operator directive: autonomous continuation (supersedes the per-command approval loop for routine work)

Operator (chat, verbatim excerpt): "I set the claude session for UAR to bypass permissions so this can run autonomously, but that is not
working. Fix that." … "This supersedes the older AGENTS.md/CLAUDE.md per-edit exact-command approval loop for routine, in-scope UAR
build, diagnostic probe, causal repair, and focused verification." … "Continue the frozen R1–R23 completion-first qualification
autonomously after routine edits, recording source/binary identity, bounded results, and cumulative calls without stopping for another
command approval each time." … "Do not change Claude permission controls, expand scope, release, deploy, switch providers, or incur new
spending. Stop only for a genuinely new safety/access decision or an unanticipated material action."
Lead interpretation:
- No Claude permission setting or settings file is changed.
- AGENTS.md is not edited (operator prose).
- From now on, routine in-scope builds/probes/repairs/focused verification proceed without per-command approval pauses.
- Kept: serial builds, single writers, candidate-specific grades, source/binary identity on every build, honest NOT READY, and
  security/trust boundaries.
- Still stops for: a new security, access or trust decision; spending; a provider switch; release/deploy; scope expansion.
The in-flight approved R9-fix build (source-scoped 629eea07) and its one R9 probe continue unchanged. Count ~1901/2232.

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

## Addendum 2026-10-20T09:03:49Z — R2(d) projector repair authored (owner uar-runtime, Sonnet 5.5 inherited; 2+1 self-reported, 3 observed)

`Projector::project` gains one arm: `NormalizedEvent::SkillActivated` -> AG-UI `CUSTOM` `uar.skill.activated`, with value `{skill:{id,title},selectionMethod}` mirroring `adapters.rs:134-146` (the shape is mirrored because the adapter closure is not callable); top-level `threadId`/`runId` (wire ids, as RUN_STARTED/RUN_FINISHED use) and `agentId` = the binding `row.agent_id` (the route agent; added for the frozen agent-identity requirement). Ordering, cursor/index scheme, terminal logic, the R9 fix and the diagnostics are unchanged. Unbuilt. New source identity: agui.rs `90cc1b4caea777cf727ca59e9fbfb3d19f3f58fa3b15f5b762b83ff865d30b4e`; source-scoped `3a874787a1a0ca93ca322337254df618194f3f1ee3f8f4831b00e30793783751`. Build + one full rows-b run (R7, R9, R2(d), R10) on the resulting sidecar proceed autonomously (directive 08:56:19Z).

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

## Addendum 2026-10-20T09:30Z — R22 child-class plan (lead source inspection) + roster

Read-only lead inspection (`cost_budget.rs:1500-1545` begin_call; `persistence/providers/accounting/surreal.rs` admit/usage/checked;
`accounting.rs:191-211` reserve; `manager.rs:736-770,5655-5700`):
- Durable admission is optimistic: read scopes -> compute `after` -> one transaction `UPDATE ... WHERE data=$before` per scope,
  with THROW 'accounting_revision_conflict' on a miss. There is no re-read/retry. Only the `accounting_revision_conflict` text is
  mapped (to `AccountingError::Conflict`); any other surrealdb error passes through, untyped.
- The observed r22c cause `accounting_admission_unavailable` is the catch-all for a non-AccountingError. It occurred while two
  children shared the root scope (`rows-a/stderr-rows.log`).
- **Hypothesis, NOT observed:** a SurrealKV write-write TransactionConflict (the same class as R9).
- Per the R9 precedent, step 1 is diagnostics only: a closed, field-free `storage_error_kind` log field in begin_call.
  - Writer: uar-runtime, fresh (Sonnet 5.5, inherited from `.claude/agents/uar-runtime.md`).
  - Single file: `src/uar/runtime/cost_budget.rs`. Source only.
  - The lead builds after the in-flight R21 harness run, then reruns R22 harness A.
- Separate, independent gap: the client sees no budget class on admission refusals. The fix is deferred until the mechanism above is
  known, so that one change does not hide the other.
Roster now: uar-state (fresh, Sonnet 5.5 inherited; `uar-r14/r21-sidecar/`, R21 memory leg, at most 2 runs); uar-runtime (fresh,
Sonnet 5.5 inherited; cost_budget.rs diagnostic); lead uar-lead (Opus 5.5, observed). Count ~1937/2232.

## Addendum 2026-10-20T09:34Z — R22 diagnostic authored (uar-runtime, Sonnet 5.5 inherited; 7 observed)

`src/uar/runtime/cost_budget.rs` (sha256 `5480e165…b68e`):
- A private `storage_error_kind(&anyhow::Error) -> &'static str` walks `error.chain()` and returns a closed label:
  - `transaction_conflict` — a typed `surrealdb::types::QueryError::TransactionConflict`, the same match as the R9 fix;
  - `other_storage_error`;
  - `accounting_error`;
  - `non_storage`.
- The surrealdb arm is `#[cfg(feature = "surreal-backend")]`. The lead confirmed `server-full` ⊇ `minimal` ⊇ `surreal-backend`
  (`Cargo.toml:223-224`).
- The label is added to the existing admission-refusal `tracing::error!`. Control flow, errors, retries and client behaviour are
  unchanged.
- Lead review of the diff: matches the spec; no error text is logged.
- Recorded limits:
  - a CAS miss that THROWs `accounting_revision_conflict` reads as `accounting_error` (already visible in `cause`);
  - `checked()` reports only the lowest-index statement error.
Unbuilt. The build waits until the in-flight R21 sidecar harness finishes (one runtime/CPU lane). Count ~1940/2232.

## Addendum 2026-10-20T09:52Z — R21 memory leg on sidecar 96e5b1f6: demonstrated product defect; causal repair queued

Harness by uar-state (Sonnet 5.5 inherited; 46 observed): `uar-r14/r21-sidecar/` (r21sc-probe.sh, r21sc-assert.py, mock_llm_r21sc.py).
- **Run:** one boot. Boot sha256 96e5b1f6 matched; ports 1919/50159/18919; synthetic sentinels; store deleted.
- **Re-analysis:** the teammate fixed an assert-script H6 regex bug and re-analysed the SAME raw outputs, with no second boot.
  Run-1 outputs and the buggy assert are preserved in `attempt1/`. The R21 repair run is still unspent.
- **Window:** W=8209 (F=3250: 1869 tool definitions + 381 world state + 1000 reserved output; X=459; history room 4500).
- **Leg A (the literal criterion: memory_enabled on every turn):**
  - T1-T6 PASS: system msg 0; RAG codeword in the system message; both skills present; trimming starts at turn 3, oldest first,
    sliding_window, recorded in context_update events.
  - T7b PASS: the final body is ~6741 tokens + 1000 reserve <= 8209, by a chars/4 estimate. Dispatched, not refused.
  - **T7a FAIL:** no `[MEMORY CONTEXT]` / retrieved.memory fragment from turn 2 onward; reply `NO-RECALL`.
- **Leg B (teammate addition; memory off for H1-H5):** T1-T7 PASS. The seed survives the H6 trim; reply `RECALL:`.
- **Lead confirmation of the mechanism:**
  - Source: `src/uar/memory/context_builder.rs:94-95` breaks at the first ranked record that exceeds the cap
    `min(max_context_tokens, profile.context_window/10)`, i.e. 800 tokens for the default profile.
  - Data: `out/legA-mem-after-t2.json` holds the 14-token seed plus 1099-token session auto-captures.
  - An empty block with a fitting seed present is only possible when a non-fitting record ranks first (leg B shows the search does
    return the seed). The exact rank order at turn 2 is inferred.
**Grade R21 on 96e5b1f6: FAIL (demonstrated defect: memory is dropped by the memory-block budget loop, not by the strategy).** The
grades of record are unchanged: v4 FAIL; f1c2f473 PARTIAL.
**Causal repair (autonomous, routine):** `break` -> `continue` (greedy fill in rank order).
- Writer: uar-state, resumed, single writer of `context_builder.rs`, source only.
- It is batched with the R22 cost_budget.rs diagnostic into ONE sidecar build.
- That build is followed by the R21 repair run (leg A) and the R22 harness-A rerun on the resulting candidate.
Recorded but not fixed (out of the observed failure):
- the memory cap uses the static `profile_for(model_id)` window, not the declared provider window;
- the `messages_removed` counts do not match the observed drops (meaning unconfirmed).
Count ~1948/2232.

## Correction 2026-10-20T09:46Z — addendum timestamps

The headings "09:16Z", "09:30Z", "09:34Z" and "09:52Z" above were lead estimates, not clock readings. "09:52Z" is
later than the real time of writing. Observed clock times: the rows-b full run ran 09:08:38-09:09:54Z, and the c5 build ran
09:31:40-09:34:16Z. From here on, headings carry `date -u` readings.

## 2026-10-20T09:46Z — combined build c5 + R21 repair-run status

- **Build** (autonomous; cached debug sidecar bin; lead-run): 09:31:40Z -> 09:34:16Z, **exit 0, 0 warnings**.
  - **New sidecar candidate `ecd3986589891f9810912159fe793327b5738a9153e8f0b23f64c6cf7827bfe3`**
    (`uar-r14/c5/uar-sidecar-c5.bin`, read-only; `c5/identity.txt`).
  - Source-scoped identity `b29ad403…` (context_builder 75c7daec `break`->`continue`; cost_budget 5480e165 diagnostic; agui
    90cc1b4c; surreal f27797ee); HEAD 38f0b5d8.
  - Lead diff review of context_builder: matches the spec.
- **R21 repair run, attempt (run 2): harness hang, NOT a product result.**
  - The probe stopped after setup (policy/provider/KB/agent-patch all 200/201) at its message-generation step, before any R21
    turn. `out/msgs` was empty for more than 6 minutes and the sidecar was idle.
  - Stopped by the lead: own processes only; the sidecar needed SIGKILL after SIGTERM; 1906/1907/1908 untouched; ports
    1919/18919/50159 freed.
  - Preserved: `r21-sidecar/run2-hung/`.
  - Cause not identified. The only difference from the teammate's run 1 is the invocation (the lead's stdin was the tool
    socket).
  - It is not counted as the repair run, because no R21 behaviour was exercised.
- **Run 3:** same probe, `</dev/null` stdin, started 09:45Z.
Count ~1966/2232.

## 2026-10-20T09:46:41Z — R21 repair run (run 3) on c5 `ecd39865`: R21 PASS (A)\* there

Run 09:44:19Z -> 09:45:40Z, `bash ./r21sc-probe.sh </dev/null`; boot sha256 = ecd39865 (lead-fixed expected); 31 s; launch token absent;
store and home deleted. Receipts: `uar-r14/r21-sidecar/run3.console.txt`, `out/`. **16/16 assertions PASS** (legs A and B, T1-T7).
- Fixture: F=3254, X=463, W=8217.
- **Leg A, the literal criterion (memory_enabled on every turn):**
  - The memory block (2-3 records, about 36 tokens, with the memory codeword) is present in every dispatched turn from 1 to 6,
    including every turn after trimming starts at turn 3.
  - Final reply: `SOURCE:<kb codeword> RECALL:<memory codeword>`, decided from the RAG block plus `[MEMORY CONTEXT]`.
  - On 96e5b1f6, before the fix, the block was absent from turn 2 onward.
- **Trim order:** sliding_window, oldest first. Survivors at turn 6 are H3, H4 and H5; SEED, H1 and H2 were dropped. Recorded in
  context_update events at turns 3-6.
- **Protected items:** system prompt (msg 0), RAG (system message), both skills (prompt and tools field) and memory are preserved.
- **Fit:** the final body is about 6789 tokens + 1000 reserve = 7789, against W=8217 (margin 428). The runtime dispatched it, with no
  context_overflow.
**R21 on ecd39865: PASS (A)\*.** Flags:
- mock model;
- token counts are chars/4 estimates (a chars/3 estimate of the body, 9052, would exceed W), so the fit rests on the runtime's
  own counter, as for T1 on f1c2f473;
- the memory cap still uses the static default profile window (recorded, not fixed).
Grades of record: v4 R21 FAIL; f1c2f473 R21 PARTIAL; 96e5b1f6 R21 FAIL. The v4 tally is unchanged (9/6/5/2/0/1, NOT READY).
The c5 candidate has no other row receipts yet; the R22 harness-A rerun on c5 is in flight. Count ~1975/2232.

## 2026-10-20T09:48:57Z — R22 harness A on c5 `ecd39865`: R22 PARTIAL there; diagnostic result

Run 09:46:19Z -> 09:47:20Z, `bash ./rows-a.sh </dev/null` (fixture-only repoint IDD/BIN to c5; backup `rows-a.pre-c5.sh.bak`;
the prior a38c3169 outputs are preserved read-only in `rows-a/run-a38c3169/`). Boot sha256 ecd39865. Log: `rows-a/rows-a-c5.log`.
Result lines: 10 PASS, 2 FAIL (the harness SUMMARY printed FAILS=1; the counter discrepancy is recorded and the lead counts
the FAIL lines).
- R3 A1-A3 PASS (configured tool, trace, result upstream, reply from result).
- **R22 root tool-call class PASS:** the 2nd echo is refused before execution, with the class named in the tool result and events.
- **R22 child limit, enforced before the action:** C2 PASS (no 5th child created or model-requested) and C3 PASS (4 children
  spawned; the 5th approved, then refused by the limiter).
  - On a38c3169 the same harness spawned only 1 child (coordinator stalled; prior catch-all admission cause). That failure mode
    did not recur on c5.
- **C1 FAIL — class not reported:** the 5th spawn's tool result is
  `{"status":"error",...,"message":"agent host operation failed"}`, with no budget class. Lead-extracted from `r22c-events.json`.
- **AGG1 FAIL — aggregation not demonstrated:** the budgeted coordinator (30000 turn/session tokens) was refused at its FIRST model
  admission.
  - Diagnostic log: `cause=accounting_usage_unresolved`, `storage_error_kind=accounting_error`, `reserved_tokens=None`,
    `reservation_source=catalog_context_and_output`.
  - The mock model is not in the catalog, so liability is unknown and the token budget fails closed. That refusal is truthful;
    the client sees no class.
- **Diagnostic outcome:** no `transaction_conflict` or `other_storage_error` was logged. The earlier r22c catch-all cause was not
  reproduced on c5, so the TransactionConflict hypothesis is neither confirmed nor refuted. No retry is introduced.
**R22 on c5: PARTIAL.**
- PASS: root tool-call class; child limit enforced before the action.
- FAIL: the class is not reported on child-limit refusals or on model-admission refusals; aggregation is not demonstrated, because
  the fixture model has unknown liability.
- Earlier, v4 only: input+reserve class PASS.
- Not configurable (recorded): AgentTreeLimits.
**Next (autonomous causal repair, observed failures C1 and admission class):** report a closed, field-free budget/limit class to the
client on (1) spawn-limit refusals and (2) model-admission refusals.
- Writer: uar-runtime (resumed; Sonnet 5.5 inherited). The files are named in its handoff before editing; no concurrent writers.
- The aggregation fixture needs a liability-resolvable model, either a catalog-known model id on the loopback mock or a declared
  output ceiling. This is evaluated separately.
Count ~1986/2232.

## 2026-10-20T10:02:50Z — c6 build + R22 rerun with the aggregation fixture; findings

**R22 class-reporting repair** (uar-runtime, resumed; Sonnet 5.5 inherited; ~22 calls):
- `control.rs` 648b1eda: `AgentControlError::Host` displays a closed label for AgentLimitError
  (`agent_tree_limit_exceeded: max_concurrent_children|max_depth|max_total_children`), else the old text.
- `cost_budget.rs` dbc9dbb8: `client_failure` maps a BudgetFailureOrigin category + closed AccountingError to a client code and
  message.
- `orchestrator.rs` 0b1e3bad: provider_failure and fallback-exhausted use client_failure, not `provider_error`.
- Lead diff review: closed labels only.
**c6 build:** 09:53:43Z -> 09:56:33Z, exit 0, 0 warnings. **Sidecar **
(`uar-r14/c6/`); source-scoped 3f9b63d6…; HEAD 38f0b5d8.
**Read-only research** (Explore, Sonnet requested, 15 calls) — **product finding:**
- No production run path attaches `LlmRequest.budget_contract`. The only Some() is `DestinationRequestPreparations`, wired only in
  tests, and team profiles clear it.
- So durable token reservation uses only the compiled models.dev catalog. For any custom-provider model, a token budget refuses
  every call as `accounting_usage_unresolved` (truthful fail-closed, but token budgets are unusable on custom providers).
- A fix that reads provider-declared limits would change how admission trusts operator metadata (team_profile.rs comment: operator
  metadata cannot prove gateway capacity). **That is a trust decision for the operator, not taken here.**
- Also found: `enrich_provider_config` (registry.rs:735-760) copies a catalog provider's `auth.env_var` from the process env into
  an empty api_key. The sidecar runs under `env -i` with only the synthetic LLM_API_KEY, so no real key could be copied. The fixture
  now also sets an explicit synthetic api_key.
**Aggregation fixture** (lead, fixture-only; backups `rows-a.pre-agg.sh.bak`, `mock_llm_rows.pre-agg.py.bak`):
- a loopback provider `mistral` with model `open-mistral-7b` (catalog 8000+8000 = 16000-token liability);
- `UAR_LOCAL_ONLY=true` (the boot log confirmed "every enabled provider is loopback");
- r22a-coord and its child on that provider.
**R22 run on c6** (09:59:21Z -> 10:00:28Z; `rows-a/rows-a-c6.log`):
- PASS: R3 A1-A3; R22-R1/R2 (root tool class); AGG0 (control).
- Liability now resolves (reserved_tokens Some(16000)).
- **The coordinator's 2nd admission (child spawned and pending) was refused with `cause=accounting_admission_unavailable`,
  `storage_error_kind=other_storage_error`.** This reproduces the a38c3169 r22c pattern (parent admission while children pending).
  The error is a surrealdb error that is NOT TransactionConflict. The client now gets code `budget_admission_failed` (the repair
  works), with no further cause.
- **Fixture defect:** moving the shared `r22-child` to the aliased provider made all 5 r22c spawns fail with the generic host
  error. The likely cause is a parent/child provider mismatch (UNCONFIRMED). Fixed by a separate `r22a-child` artifact for the
  budgeted chain only; r22-child is back on r4mock.
**R22 on c6: not graded.** The run is confounded by the fixture defect for C1-C3; the aggregation leg is blocked by the storage
error.
Next:
- diagnostic step 2: `storage_error_detail` = closed surrealdb `kind_str()` of the error and its innermost cause (writer
  uar-runtime, cost_budget.rs only);
- then build c7 and rerun rows-a.
Count ~2030/2232. **Ledger extension:** 2232 -> 2532 recorded now, before crossing (same rationale as #6-#10: remaining R22,
consolidation re-proofs).

Correction (same entry): the c6 sidecar sha256 was dropped from the line above by shell expansion. It is 66ccf1dc3efc6089a88ecac7126bc9385ef7694eb662d3ca7085989bee937a2b (`uar-r14/c6/expected.sha256`).

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

## Pre-reboot receipt preservation (operator request: copy uar-r14 into the KBD receipts area, no commit)

- Destination: `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/receipts/scratch-uar-r14-2026-10-20/` (5.5G, UNTRACKED, not
  committed).
- Copied with `rsync -a`, 10:18:27Z -> 10:19:50Z, exit 0.
- **Excluded on purpose:**
  1. `.credkey`: a credential key, never read and never placed in a version-controlled tree.
  2. `data/`: the LIVE SurrealKV store of pid 24103 (port 1907, the v4-lane instance started with `uar-r14/config.yaml`; it holds
     `data/phase.db/LOCK` and its WAL open). A live copy would be inconsistent, and the store likely holds that instance's encrypted
     provider credentials.
- **Verification:**
  - 1636 source files = 1636 copied files.
  - A checksum dry run (`rsync -anc`, excluding the still-open `boot-v4b.log`) shows 0 differing files.
  - The candidate binaries' sha256 match their recorded `expected.sha256`: c5, c6, c7, r2dfix, r9fix, r9diag, identity.
- **Hazards for the operator:**
  - The copy includes about 5.4G of debug binaries inside a git-tracked tree and is not gitignored. Do not `git add -A`/commit it
    as-is.
  - The 1907 instance's store and key remain ONLY in the macOS temp dir. A reboot stops 1907, and a temp cleanup would lose that
    state.
