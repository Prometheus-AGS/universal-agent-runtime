# OpenSpec phase-close verification — 2026-09-04

Scope: the eight active changes in the ten-change
`codex-harness-comparative-analysis` plan. `context-history-integrity` and
`project-instructions-world-state` already have archived records. This report
does not archive, publish, certify a deployment, or mark the wider goal complete.

## Delta between plan and delivery

All ten changes are implementation-complete in canonical revision2298. The
corrected full server-full phase command passed. One OpenSpec receipt remains
unchecked: model-path-resiliency5.4, the proposal's explicitly deferred real
provider429 observation. Tests with controlled429 responses do not satisfy it.
Feature-gated persistence tests and several live boundary scenarios below are
also not established by this matrix. These limits preclude an unconditional
"all scenarios verified" statement despite the green suite.

## Method and scorecard

Ran `openspec status --change <name> --json` and
`openspec instructions apply --change <name> --json` for each named change.
Every action context is repo-local and scoped to this checkout. Read the
returned proposals, tasks and delta specs; mapped requirements to production
callers and scenario tests. No change has a design.md, so formal design-file
adherence was skipped; coherence was checked against the proposals, staged
dependency plan, source contracts and recorded decisions instead. OpenSpec's
artifact `isComplete: false` is not the implementation counter.

| Change | OpenSpec tasks | Requirement implementation mapping | Coherence |
|---|---:|---:|---|
| deterministic-prompt-assembly | 18/18 | 4/4 located | Fixed sections and redacted metadata |
| fail-closed-tool-arguments | 20/20 | 5/5 located | Host admission retained; effects, not names |
| model-path-resiliency | 22/23 | 6/6 located | Deferred live429; replay uses original owner/run |
| progressive-skill-runtime | 20/20 | 6/6 located | Catalog budget; activation cannot widen |
| typed-turn-assembly | 17/17 | 3/3 located | Seven stages; host-only capabilities |
| projected-mcp-runtime | 22/22 | 7/7 located | Peer-local catalogs; unsupported sandbox rejected |
| thread-native-subagents | 26/26 | 8/8 located | Persisted threads; narrow-only authority |
| typed-turn-default-flip | 8/8 | 1/1 located | Recorded evidence precedes default flip |

Total active OpenSpec checkboxes:153/154. These are the current files' counts,
not the canonical task-history counts, which retain duplicate historical rows.
Requirement mappings are static implementation evidence, not40 independent
runtime proofs. Canonical implementation remains10/10, overall111/120.

## Requirement and scenario evidence

Paths below are repository-relative. Test names and source locations were
searched in the current checkout, not inferred from the old proposal line numbers.

### deterministic-prompt-assembly

- Typed fixed ordering and explicit authority: `src/uar/runtime/prompt/assemble.rs`
  (`PromptSection`, `render_with_options`) and `fragment.rs`; manager constructs
  real fragments at `src/uar/runtime/manager.rs:2488`. `tests/prompt_assembly.rs:129`
  tests discovery-order independence, :139 authority/markers, and :244 stable
  successive-turn prefix snapshots.
- Redacted manifest: `prompt/manifest.rs` has metadata-only `ManifestFragment`;
  manager constructs `TurnManifest` at :3534. Tests :168 check body exclusion
  and :205 check stored manifest plus both emitted artifacts.
- Artifact instructions: the host section and ordering are exercised at
  `tests/prompt_assembly.rs:309`. Six target tests passed.

### fail-closed-tool-arguments

- Strict argument parsing/schema validation and once-compiled validators:
  `src/uar/tools/validate.rs:38`, :49, :69 and descriptor validator at
  `src/uar/tools/descriptor.rs:101`; `tests/tool_call_protocol.rs:138`, :186,
  :1021 exercise malformed, schema-invalid and repeated-call cases.
- Declared effects and approval: descriptor enums at :28/:41;
  `tests/tool_call_protocol.rs:806` verifies an MCP read-only hint does not
  bypass Ask. The manager's host gate remains before dispatch.
- Parallel scheduling: `src/llm/orchestrator.rs` uses host admission receipts
  with read/write effects and ordered results. Tests :371/:406 cover compatible
  and shared keys; :698 covers the actual governed manager; :732 verifies
  confirmation ordering and single budget charge; :783 rejects over-budget work.
- Collisions/namespacing: `ToolCollision` at descriptor.rs:64, assembled registry
  descriptors and test :969. Eleven target tests passed. Approval rejection and
  timeout UI/HTTP scenarios are not independently established by this target;
  do not equate positive approval coverage with the entire inherited workflow.

### model-path-resiliency

- Policy retry and typed errors: `src/llm/provider_error.rs:25`, :95, :130;
  orchestrator retry path and `src/uar/settings/resilience_policy.rs`.
  Tests at `tests/model_path_resiliency.rs:329`, :373, :404, :416, :712
  exercise jitter, real driver HTTP classification, Retry-After on/off and
  non-retryable kinds. Real-provider429 remains deferred.
- Health-gated selection: `src/uar/runtime/manager.rs:2934`, model router and
  orchestrator health attachment; test :428 verifies a cooled fallback is not
  attempted. That is not a separate proof of every primary-cooldown permutation.
- Idle timeout and interrupted persistence: test :504, :570, :629, :675,
  :783 cover no output, repeated metadata, cumulative usage, partial idle and
  next-turn interrupted history. Twelve target tests passed.
- Original-run chat replay: `src/server.rs:4893` cursor and :5390 retained
  history branch; `tests/integration/live/chat_replay_cases.rs` exercises both
  aliases, four formats, mid-event frame cursors, legacy and terminal cursors,
  cross-tenant rejection, malformed/future/mismatched cursors, expired prefixes,
  and no second primary model call. It passed separately and in the full matrix.
  Enabled memory/quality side effects and cancelled-run replay are not covered.

### progressive-skill-runtime

- Budgeted catalog: `src/uar/runtime/skills/catalog.rs:122`, :212; tests at
  `tests/skill_activation_runtime.rs:282` and :339 prove2000 nonempty titles
  fit the chosen cap and extreme omission preserves retained metadata.
- Explicit activation and narrow eligibility: `skills/activation.rs:362`,
  its projected host and next-step dispatch; tests :370, :435, :570 verify
  attachment, model activation, missing/disabled rejection and max_active.
- Implicit ranking/modes: `skills/matching.rs`, service matching and test :624.
  The default remains legacy_overlay; this phase did not authorize a catalog
  default flip without recall evidence.
- Shadow reduction: activation.rs:341/:495 retains candidates and emits recall
  without changing the authorized catalog. No dedicated explicit-activation
  recall-miss regression was identified in the named target; this scenario
  remains a coverage warning, not a claim of99% measured recall.
- Retention and attribution: tests :676/:752 cover compaction reattachment and
  prompt-only/multi-skill attribution without doubling aggregate totals.
  Eight activation tests and three scoped-governance tests passed.

### typed-turn-assembly

- Immutable snapshots: `src/uar/runtime/turn/request.rs`, `plan.rs`, `resolved.rs`;
  production `ResolvedStep::new` at `src/llm/orchestrator.rs:1126`.
  `tests/typed_turn_assembly.rs:122` compares old/owned entry adapters; skill
  activation and MCP discovery targets exercise next-step changes.
- Staged narrow-only contribution: `turn/contributors.rs` executes seven fixed
  stages, validates authorized descriptor equality and rejects policy widening;
  tests :174/:204 cover unauthorized tools and direct-entry memory. Three passed.
- Shadow parity: `turn/shadow.rs:121` and :166 compare turn/step views;
  `tests/turn_shadow_parity.rs:69` passed. Corpus is three requests, zero
  unexpected differences. The allowlist is not evidence of broad combination
  coverage. The later default-flip change supersedes this change's legacy default.

### projected-mcp-runtime

- Separate immutable definitions, authority and exact projections:
  `src/mcp/catalog.rs:96`, `projection.rs:173` and :252, root capture at
  `src/uar/runtime/manager.rs:858`; test `tests/mcp_projection.rs:725` covers
  global precedence and host-local delegation recipes.
- Reuse/invalidation/single-flight: `binding_cache.rs:100`, :451, :585, :784;
  tests :286/:928 cover reuse/config change and cancelled refresh. Key fields
  include auth identity, but a live credential-rotation case is not established
  by the config-change test.
- Lazy readiness: `runtime.rs:287`, :389; test :408 and real stdio test :1258.
- Required/optional failure: `preflight.rs` and test :528.
- Bounded exposure/search: `exposure.rs:79`, :139 and per-step orchestrator
  integration; test :610 verifies next-step-only discovery.
- Secret-free lifecycle: `lifecycle.rs`, registry publication and test :1084.
  A live expired-credential authentication exchange is not proved by ordered
  lifecycle tests alone.
- Sandbox flag: configuration validation and test :1223 reject unsupported
  sandboxed stdio before launch. This implements the specified rejection option,
  not an OS sandbox. Nine tests passed; real stdio receipt is in this change's
  `evidence/stdio-integration.md`.

### thread-native-subagents

- Persisted shared kernel and graph delegation: `src/uar/runtime/thread/service.rs`,
  `kernel.rs`, manager attachment at :3097, actor and graph adapters;
  `tests/agent_threads.rs:503`, graph targets and named A2A test
  `tests/a2a_thread_service.rs:214`. Memory/PostgreSQL provider tests at
  agent_threads.rs:489/:533 are excluded by server-full; PostgreSQL additionally
  requires DATABASE_URL and can early-return without a live connection.
- Narrow policy and root approval: `thread/policy_intersection.rs`, including
  named-child mode correction at :747; `tests/agent_policy_intersection.rs:97`,
  :142/:171 cover denial, unsupported shapes and root approval authority.
- Typed messages: `thread/messages.rs` and agent_threads.rs:357. Lifecycle
  content exclusion and AG-UI mappings: agent_threads.rs:401.
- Limits/budget/cancellation: `thread/limits.rs`, `cost_budget.rs` and service;
  agent_threads.rs:195/:229/:254 cover limits. New host-path tests in
  `thread/service_tests.rs` cover never-dispatched reservation release and joined
  shutdown; their exact limits are recorded in `evidence/remote-host-regressions.md`.
- Explicit agent-control authorization: `runtime/native_skills/agents/` and
  root service registration. The named targets do not independently prove an
  entire unauthorized-spawn model interaction.
- Inbound A2A and actor auth: a2a_thread_service.rs:214/:276/:343 cover named
  run/get, cancel and401 for unauthenticated actor endpoints. Three passed;
  eight thread tests and four policy tests passed under server-full.
- Real-model cancellation receipt covers a local child awaiting its first
  provider response. It does not prove live remote-peer enforcement/cancellation,
  after-text cancellation, physical inference termination or billing termination.

### typed-turn-default-flip

- Config default and rollback: `src/config.rs:200`, test :1907 and settings
  schema at `src/uar/settings/manager.rs:1239`; release guidance is in
  `docs/releases/typed-turn-default.md`.
- Three-case parity report plus two-case real k3 shadow receipt precede the
  flip in the retained decision/evidence records. Both have zero unexpected
  differences. The default/legacy test passed in the corrected full suite.
- Missing-evidence behavior is a pre-merge workflow gate, not runtime filesystem
  inspection. No claim that removing an evidence file changes an installed default.

## Observed commands and results

`openspec validate <name> --strict` printed `Change '<name>' is valid` for all
eight named changes; each invocation completed in the sequential read-only batch.
This validates spec structure, not runtime behavior.

The final phase command and actual output are retained in
`audit-correction-report.md`: locked server-full check, formatting check, then
full tests; exit0; library713 passed/1 ignored, BDD9 scenarios/49 steps passed,
broad integration94 passed/1 ignored, doctests26 passed/17 ignored. All executed
targets passed. No tests were rerun for this documentation-only closeout pass.

## Findings and disposition

CRITICAL checklist exception before unconditional archive: model-path-resiliency
task5.4 is incomplete. The proposal explicitly defers it, so it is not an
implementation defect. Preserve the unchecked item; obtain explicit approval to
archive with this exception, or record a genuine provider429 observation first.
Do not manufacture429 traffic or mark a controlled response as live evidence.

WARNING — scenario coverage: the specific unproved scenarios above remain
limits. Before certifying those behaviors, add/run the named boundary coverage:
memory/PostgreSQL provider variants; live peer cancellation; rejection/timeout
approval flow; primary cooldown; recall miss; credential rotation/auth-required;
enabled replay side effects and cancelled-run HTTP replay. No unrelated code
hardening is justified merely by an untested scenario.

WARNING — formal QA: no per-change artifact-refiner logs exist for these ten
changes and its required code-interpreter/e2b execution tools are unavailable.
Formal refiner QA is skipped; recorded independent source reviews and local
tests are the fallback. No refiner pass rate or iteration count is fabricated.

No new production defect was identified by this closeout mapping. No design.md
was available to verify. No unrelated additions, new guards or dependency changes
were made. Seven active changes have no incomplete task; the eighth requires
explicit deferred-receipt disposition. Sync/archive approval remains outstanding.
Only after that gate may kbd-reflect close this phase and planning resume for
the parent select-and-observe-presentations work.

## Independent artifact review

An isolated read-only critic checked this report and its referenced evidence.
It confirmed153/154 tasks,40 requirement mappings, the retained suite totals,
three corpus/two live parity cases, disclosed feature/live/QA limits and the
outstanding archive authorization. No concrete blocker was found. The critic
ran no tests/builds and did not independently reproduce the full suite.

## Read-only archive preflight — 2026-09-04

Canonical revision2299 still awaits approval. The eight active changes contain
ten delta files for nine capabilities. Only `turn-assembly-kernel` is shared:
apply typed-turn-assembly before typed-turn-default-flip, following their explicit
dependency and the observed `HarnessMode::Typed` default in src/config.rs.
The earlier sentence requiring legacy "in this change" is a migration-stage
constraint. When syncing, make that historical scope explicit and reference the
later evidence-gated default requirement; do not leave two contradictory current
defaults, remove the rollback, or alter the archived historical delta.

Seven capability specs do not yet exist in openspec/specs; create them during
the approved sync. Two existing specs must be merged surgically:
tool-approval-workflow retains all four unrelated UI, visual and replay
requirements while extending descriptor-based admission and the configured
timeout scenario; multi-agent-orchestration retains its purpose while replacing
the delegated-answer requirement and adding tool-use coverage from the delta.

No main spec, archive directory, source, test or canonical phase state was changed
by this preflight. The remaining action still requires explicit batch approval,
including the unchecked model5.4 receipt and missing-design/coverage warnings.
