# Coverage matrix

Planning coverage, not executed results. All25 user families and all14 historical supporting obligations must map to registry entries during implementation. Existing97case IDs are migration inputs, not a mandatory all-at-once runner shape. Every removed duplicate has an explicit semantic-equivalence mapping; no advertised route, strategy or behavior disappears.

| Group | Scope | Legacy families | Prerequisites | Inputs/journey | Observable success |
|---|---|---|---|---|---|
| G01 | Configured prompting and session continuation | U01,U02 | One real provider/model, one store, ingress | Fresh task with independently calculated answer; follow-up recalls a generated fact after multiple calls | Useful genuine output, correct session/agent/model correlation and continuity |
| G02 | Supported interfaces and primary clients | U05,U08,U24 | G01 for same platform/backend; adapter-specific client only | Default and saved custom agent through mounted native, OpenAI, Anthropic, ACP, A2A HTTP/gRPC, AG-UI/A2UI and shipped UI/SDK paths | Protocol framing/order, incremental chunks, terminal result and useful answer; A2UI interaction continuation |
| G03 | Durable agents and protected built-ins | U03,U04 | Store and agent API | Create/update/restart/prompt a custom definition; attempt default/orchestrator deletion | Definition remains durable and effective; both built-in deletions refused and agents remain usable |
| G04 | Orchestrator intent and selection stream | U06,U07 | G01, isolated registered-agent roster | Distinct specialist agents; intents, roster change, ambiguous/no-match and follow-up requests | Chosen agent follows designed classifier/roster policy; selection chunks visible on AG-UI and execution returns useful result |
| G05 | Skill discovery, strategies and governance | U09,U10,U11 | Strategy-specific tiny local skill pack | Exercise each configured detection/matching strategy, scope and disallowed activation | Actual selected skill affects execution; wrong-scope activation refused; inspectable strategy evidence |
| G06 | Context and whole-request budgets | U12,U13 | Configured strategies and token-aware request observation | Multi-call context with known facts, constrained budgets and oversized input | Designed retention/compaction and final-wire budget behavior; required facts retained or explicit designed refusal |
| G07 | Grounded KB and embedding regeneration | U14,U15 | Owned corpus/store; real admitted embedding bindings | Ingest/query grounded facts; same-width model switch, different-width switch, interrupted regeneration and restart | Answer attributable to corpus; stale vectors not silently reused; regeneration and recovery restore usable retrieval |
| G08 | Store parity and restart durability | U16 | Each adapter independently available | Equivalent CRUD/prompt/session/KB/config operations on Postgres+pgvector and disk Surreal; restart and multi-call | Equivalent user outcomes and durable reload; failure on one adapter never prevents testing the other |
| G09 | Teams and threaded context continuity | U17,U18,U19 | Real team definitions and model; team tracing | Dependent and independent tasks, overlap, follow-up, cancellation and recovery | Correct dependency order/actual designed concurrency; isolated threads; outputs reassembled into later useful context |
| G10 | REST configuration and startup maintenance | U20,U21 | Owned config database and runtime artifact | Set/read/restart config; initial/warm boot, post-init maintenance and migration cases | Effective settings persist and apply; maintenance converges without overwriting operator settings |
| G11 | Tools, approvals and owner isolation | U22,U23 | Named local tool/MCP fixture and isolated users | Tool success/refusal, approval boundary, cancellation, recovery and cross-owner attempts | Designed tool effect/refusal, isolation and honest recovery; no borrowed run evidence |
| G12 | Memory, structured formats and routing promises | U25 | Only capability-specific memory/format/route prerequisites | Persist/retrieve a fact across calls/restart; required output formats and supported routing modes | Observable durable memory, parseable required format and actual selected route |

For every row: a violated outcome is FAIL; an unmet prerequisite is BLOCKED with the exact reason. Evidence is the common append-only result record plus response/stream, persistence/restart, selected skill, budget or team trace required by that row. Owner is the corresponding capability adapter; the lead owns orchestration and migration completeness.

## Platform/backend applicability

| Platform | Required execution | Store coverage | Current evidence disposition |
|---|---|---|---|
| macOS | Actual native server and applicable shipped clients | Postgres+pgvector and disk Surreal, independently | Current-source functional cases NOT_RUN |
| Linux | Actual Linux server/artifact, container acceptable | Both adapters, independently | Current-source functional cases NOT_RUN |
| Windows | Actual Windows artifact on a Windows executor | Both supported server adapters; any genuine platform restriction requires spec evidence | Outside selected macOS→Linux→iOS sweep; historical missing-executor BLOCKED retained; no certification |
| iOS | Actual embedded SDK host on identified simulator/device | Supported embedded/host persistence; server APIs and server databases excluded only with documented design applicability | Current-source functional cases NOT_RUN; simulator evidence does not certify devices |
| Android | Follow-up after selected-platform results | Preserve previous target failure | DEFERRED; release note only after operator release decision |

Run each applicable interface at least once in each supported mode in the registry; apply three fresh positive repetitions per acceptance cell, including streaming-specific assertions. Controlled resilience scenarios run twice from independently reset owned state. Each multi-call/restart repetition is an entire journey, not one HTTP request. Match oracles across stores; do not demand byte-identical generated prose.

Build/profile/spec/Rust reviews, main acceptance and issue426 integration remain supporting obligations with explicit IDs and dependencies. A build cannot stand in for a functional cell, and missing supporting obligations remain visible. Avoid an automatic Cartesian product of protocol encodings unless the advertised contract supports that combination.

## Operator-selected sweep order

Complete all applicable G01–G12 cells and supporting checks on macOS first, then Linux, then iOS. No routine cross-platform interleaving. Terminal FAIL/BLOCKED remains visible; sweep completion does not imply PASS. Carry common fixes forward and recheck affected earlier-platform behavior after later shared edits. Windows is outside this sweep, Android deferred; neither receives acceptance credit.

## 2026-10-26 — Executed standalone Mac evidence (supersedes NOT_RUN for these narrow portions only)

Operator-directed simple BDD journeys have now verified8 behaviors across5 journeys on the retained Mac artifact. See [.prometheus/verification/macos-behavior-ledger.md](../../../.prometheus/verification/macos-behavior-ledger.md) and its97-case JSON mapping. G01–G04, G07 and narrow supporting portions of G10/G11 have recorded observations; complete family acceptance remains unclaimed. Prior platform table and repetition plan above are historical planning scope, not a denial of these actual receipts. No other platform receives acceptance credit. No rerun was performed for this reconciliation.
