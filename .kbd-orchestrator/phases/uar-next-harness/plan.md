PLAN: uar-next-harness
Project: universal-agent-runtime
Date: 2026-07-01 (amended A1, A2 2026-07-01)
OpenSpec available: YES
Changes to implement: 23 (+1 operator item, not a change)

## Amendment A2 — 2026-07-01: integration-test gate via the local OpenAI proxy

Operator requirement: "when the code builds, know that the features actually
work" — full integration coverage using the OpenAI-compatible proxy already
running for the Karpathy LLM wiki (`ai.prometheus.openai-proxy`,
`http://127.0.0.1:8181/v1`, Codex-token-backed; same endpoint pk routes via
`CLOUD_LLM_URL`, models gpt-5.4 / gpt-5.4-mini).

- **A2.1 Coverage contract (this phase's definition of done):**
  **100% FEATURE coverage, not 100% line coverage.** Every change CH-01…CH-23
  must land with ≥1 live integration case in the shared live suite (feature
  matrix, A2.3). Line coverage stays gated by the existing
  `comprehensive-tests.yml` cargo-llvm-cov threshold (80%); new code in each
  change should meet ≥80% on its own diff. 100% *line* coverage is explicitly
  rejected: the terminal ~15% is derive/boilerplate/error-plumbing whose tests
  assert nothing behavioral, and the metric is gameable (Rule 5) — operator may
  override this decision before Round 2.
- **A2.2 NEW CH-22 proxy-integration-gate (Round 1, first):** a `live`
  integration tier that boots the real server and exercises it end-to-end
  against `UAR_LLM__BASE_URL=http://127.0.0.1:8181/v1` (model
  `openai/gpt-5.4-mini`). CI-hosted runners CANNOT reach the local proxy, so
  the tier is dual-backend by design: the **same case list** runs (a) live
  against the proxy locally / pre-push / self-hosted, and (b) through the
  existing recorded-fixture provider in CI (deterministic mirror). Runner
  script health-checks the proxy first and fails with the known remediation
  (Codex re-login + `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy`)
  instead of a cryptic 401.
- **A2.3 Feature matrix:** `tests/integration/live/MATRIX.md` maps every
  CH-## → its live case(s). Baseline cases at CH-22 landing: streaming chat
  (openai + agui + dual SSE modes), tool loop w/ MCP tool round-trip, agent
  selection via `model` param, memory write→recall, RAG ingest→retrieve,
  credential-chain resolution. Each later change appends its case in the same
  PR (e.g. CH-01 gRPC task round-trip, CH-03 failover under induced 429,
  CH-04 dialect params visible in captured request, CH-16 pack detection
  precedence, CH-21 AG-UI vocabulary conformance).
- **A2.4 Relation to evals:** the eval gate (Tier-2 nightly + OP-1) checks
  *model-quality regression*; the live tier checks *feature correctness*. Both
  can share the proxy locally (`eval run` honors `UAR_LLM__BASE_URL`), but
  neither replaces the other.

## Amendment A1 — 2026-07-01: validation pass (docs/uar-next-fable.md)

`docs/uar-next-fable.md` (Fable 5) validated uar-next.md and
`model-comparison-expanded.docx.md` against the UAR, librefang, and
prometheus-skill-system codebases plus live web sources. **uar-next-fable.md
supersedes uar-next.md as the recommendation source for this phase.** Deltas
applied to this plan:

- **A1.1 → CH-03:** fix latent router bug — a model with missing cost data
  sorts as *free* (`src/llm/router.rs:75` maps `None`→`0.0`); add a
  routing-decision audit log (Rule 34); mirror librefang `fallback_chain`
  semantics for failover chains.
- **A1.2 → CH-04:** dialect requirements are the *web-verified* per-model
  params in fable §2.1: Anthropic `thinking` budgets + XML preference; OpenAI
  Responses `text.format` strict; Kimi `thinking: {type, keep}` (+ handle
  400-on-missing `reasoning_content`; Anthropic-compat endpoint exists); GLM
  `thinking_mode` high/max; Qwen `enable_thinking`/`preserve_thinking` with the
  DashScope-`extra_body` vs vLLM-`chat_template_kwargs` syntax split; MiniMax
  Markdown-aversion (prefer XML/JSON structure). Encode **no quantitative
  claims** from model-comparison-expanded.docx.md (15+ internal contradictions;
  Anthropic 2×->200K surcharge REMOVED 2026-03 — GPT-5.5 now carries a
  2×/1.5× surcharge >272K instead; tokenizer overhead is ~16% English/~30%
  code, not a flat +30%).
- **A1.3 → CH-09:** every registry entry MUST carry a source URL + retrieval
  date; use the dimension schema of fable §2.4 (effective context, dialect,
  reasoning-persistence params, cache economics, per-content-type tokenizer
  factors); populate from models.dev + provider docs, never from the
  comparison doc.
- **A1.4 → CH-16 (rescoped):** the pack is ALREADY bundled (submodule
  `crates/prometheus-skill-system`, loaded by
  `src/uar/runtime/skills/builtin_loader.rs`). CH-16 becomes **loader upgrade +
  auto-detection** (fable §6): detection precedence (env override → sibling
  checkout → installed plugin → embedded submodule; optional gated fetch), full
  agentskills.io frontmatter incl. `model_routing` → `RouteRequirements`,
  progressive disclosure (279 SKILL.md files), nested-skill hierarchy, merge of
  the pack's `.mcp.json` (7 servers, namespaced + opt-in), precedence-based
  collision policy, pack-version provenance recorded by stage s08. Do NOT
  compile skills into descriptors — skills stay portable SKILL.md; agents pin
  skill name+version.
- **A1.5 → CH-18:** first deliverable is the ZERO-CODE seam — a bossfang
  provider pointed at UAR's OpenAI-compatible endpoint via `provider_urls`
  (verified in librefang config). librefang already implements A2A
  (`librefang-runtime/src/a2a.rs`, Agent Cards) and already pins Prometheus-AGS
  `surreal-memory` (shared memory substrate); the skill bridge exists in the
  pack (`librefang-wasm-skill`, `upload-to-bossfang`). The "50+ page dashboard"
  is FALSE — `web/` is the docs site; integrate against `librefang-api`
  (140+ endpoints). CH-18 now depends on CH-21.
- **A1.6 → NEW CH-21 agui-spec-alignment (Round 4):** UAR's `agui.*` SSE names
  are invented, not the official AG-UI vocabulary (RUN_STARTED,
  TEXT_MESSAGE_CONTENT, TOOL_CALL_*, STATE_DELTA). Emit spec-conformant events
  for CopilotKit / Microsoft Agent Framework / Oracle A2UI interop.
- **A1.7 → CH-20:** `server.rs` is 4,922 LOC (not 4,848).
- **A1.8 (reflection seeds, not changes):** provider breadth is no longer a
  moat (Mastra: ~94 providers/3,300+ models behind one router); the 2026
  narrative is Hermes-style self-improving skills — a trajectory-reflection
  loop writing back to `SkillOrigin::User` skills is a future-phase candidate
  layered on CH-08. UAR's defensible intersection: signed compilation + Cedar +
  A2A/AG-UI/OpenAI-compat + Rust local-first.

## Framing (anti-sycophancy note, S-02)

The phase goal says "implement EVERY recommendation in docs/uar-next.md." The
assessment proved the doc is materially stale: 9 of its recommendations are
already implemented, and one (dependency unpinning) is actively harmful and is
**rejected** below with rationale. "Every recommendation" is therefore executed
as: every *real* gap closed, every stale row dispositioned with evidence, every
rejected row documented. Executing the doc's 20-week/8-person plan verbatim
would re-build existing systems — that conflict is surfaced, not papered over.

Scale: 21 changes is 3–4× a normal KBD phase. Execution should proceed in
**tranches mapped to nested child phases** (see Execution Round Order); each
tranche is its own /kbd-execute pass with 4–7 changes. Do not open all OpenSpec
changes up front — create each round's changes when the round starts.

## Decisions resolved in this plan (defaults chosen; override before Round 2)

- **D-A RAG (G2.6):** harden **in-process** (§5.4 mitigations: query decomposition,
  verification, audit) now; **defer** the Knowledge Service extraction (§5.3) to a
  future phase. Rationale: extraction is a 2-week re-architecture with migration
  risk; the missing capabilities can land in-process and move later.
- **D-B MemPalace (G1.5):** keep the `memory-palace` feature **off**; no active
  dependency conflict exists in the lock. Enablement is a product decision with
  its own testing burden — out of scope this phase; documented in CH-20.
- **D-C LibreFang (G3.5):** scope to the **UAR side only** (A2A task intake +
  AG-UI stream contract + example client). Full integration requires LibreFang-
  repo work by that team — cross-repo work is out of scope for this repo's phase.
- **D-D Dep pins (G1.6):** **REJECT** the doc's unpin recommendation. The
  `surrealdb "=3.0.5"` / `pgvector "=0.4.1"` pins are deliberate and load-bearing
  (client-version alignment; prevents duplicate sqlx 0.9 resolution). Action:
  document rationale (CH-20), close the doc row as won't-do.

## CHANGE LIST (ordered)

### Round 0 — Hygiene (no OpenSpec needed; direct tasks)

1. HK0-commit-live-sse-dualstack: review, test, fmt, and commit the working-tree features
   - Scope: api | frontend | hygiene
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S
   - Complexity score: Low · Model class: medium (review judgment)
   - Customer value: HIGH (fixes HTTP/1.1 6-connection exhaustion)
   - Details: The dirty tree holds real features — dual-stack companion listener
     (`src/server.rs`), multiplexed `/api/live` SSE (`src/uar/api/live.rs`),
     shared-EventSource frontend adapter + test. Run tests, `cargo fmt` (also
     `registry.rs`, `routes.rs`), commit as focused commits. Also commit the
     `.kbd-orchestrator/phases/uar-next-harness/` state and untracked `.prometheus/`
     decision (likely .gitignore).

### Round 1 — Foundation (child phase: `foundation-completion`)

1b. CH-22 proxy-integration-gate (NEW per A2): live integration tier + feature matrix
   - Scope: tests | ci | scripts
   - Depends on: HK0 (clean tree); lands BEFORE other Round-1 changes so they gate on it
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium · Model class: medium
   - Customer value: HIGH (operator's build-confidence requirement)
   - Details: See Amendment A2.2/A2.3 — dual-backend live tier (local proxy
     127.0.0.1:8181 / recorded-fixture in CI), `scripts/live-integration.sh`
     with proxy health-check + remediation message, baseline feature cases,
     `tests/integration/live/MATRIX.md`, and a per-change acceptance rule: every
     subsequent CH adds its live case in the same PR.

2. CH-01 a2a-grpc-enable: make the A2A gRPC transport compile, export, and serve
   - Scope: api | build | tests
   - Depends on: NONE
   - Recommended agent: Claude Code / Codex
   - Est. complexity: M
   - Complexity score: High · Model class: frontier (build-system + tonic 0.14 migration)
   - Customer value: HIGH
   - Details: Re-enable `tonic_build::compile_protos` in `build.rs` (tonic-build
     0.14 API), un-comment `pub mod grpc` (`a2a/mod.rs:21-23`), fix
     `include_proto!("a2a")` compile, mount the gRPC service in `server.rs`
     (currently commented at :1050-1067), add a tonic-client integration test.
     Existing OpenSpec spec: `openspec/specs/a2a-grpc`.

3. CH-02 postgres-credential-store: multi-tenant encrypted credentials on Postgres
   - Scope: db | api | tests
   - Depends on: NONE
   - Recommended agent: Claude Code / Codex
   - Est. complexity: M
   - Complexity score: Medium · Model class: medium
   - Customer value: HIGH
   - Details: Implement `PostgresCredentialStore` (parity with
     `SurrealCredentialStore`), sqlx migration, AES-256-GCM via existing
     `encryption.rs`, replace the in-memory fallback at `server.rs:188-190`,
     unit + integration tests against the compose Postgres.

4. CH-03 provider-health-failover: health-driven routing + wired failover
   - Scope: llm | api | ui(console)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High · Model class: frontier
   - Customer value: HIGH
   - Details: Consume the dead `health_check_secs` config with a provider health
     monitor loop; feed health status into `ModelRouter::route` selection; wire
     the orphaned `with_failover` API into the registry/orchestrator call path
     (static chains at `registry.rs:432-449` as fallback source); metrics +
     Runtime Console surfacing. Completes the ~80%-done router (assessment G1.3+G2.7).
     **A1.1:** also fix missing-cost-sorts-as-free (`router.rs:75`), add
     routing-decision audit log, mirror librefang `fallback_chain` semantics.

5. CH-04 prompt-dialect-engine: per-model prompt dialect transformation
   - Scope: llm | compiler-adjacent | tests
   - Depends on: NONE (extends tool_normalizer/capability_registry subsystem)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High · Model class: frontier
   - Customer value: HIGH (doc §19 headline item #2; biggest true net-new gap)
   - Details: `PromptDialectEngine` with per-model dialect profiles (XML for
     Claude-class, JSON for OpenAI-class, Markdown, GLM), dialect detection keyed
     off `ModelCapabilityRegistry`, applied in orchestrator prompt assembly.
     Explicitly extend the existing `tool_normalizer`/`xml_tool_injector`
     subsystem (assessment Q2b) rather than a parallel implementation.
     **A1.2:** requirements = verified params in uar-next-fable.md §2.1 (Kimi
     `thinking.keep`, GLM `thinking_mode`, Qwen `enable_thinking`/
     `preserve_thinking` + syntax split, OpenAI Responses `text.format`,
     MiniMax Markdown-aversion); no numbers from the comparison doc.

### Round 2 — Intelligence (child phase: `intelligence-completion`)

6. CH-05 per-model-context-strategy: model-aware chunking/placement/compression
   - Scope: runtime/context | settings
   - Depends on: CH-04 (dialect profiles provide the per-model keying)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High · Model class: frontier
   - Customer value: MEDIUM
   - Details: Implement real `Summarize`/`Hierarchical` (currently sliding-window
     fallbacks, `strategy.rs:22-23,42`), per-model strategy selection via
     catalog/capability profile, placement policy (lost-in-the-middle avoidance).

7. CH-06 cost-budgets-backend: per-task spend aggregation + budget alerts
   - Scope: runtime | metrics | events
   - Depends on: NONE
   - Recommended agent: Codex / Claude Code
   - Est. complexity: M
   - Complexity score: Medium · Model class: medium
   - Customer value: HIGH
   - Details: Aggregate per-run/per-task/per-session spend (cache-read cost too,
     `manager.rs:1424` gap), budget config (per agent/task), threshold alert
     events + metrics. Builds on existing `estimate_cost` + `uar_llm_cost_usd`.

8. CH-07 cost-dashboard: spend + budget alert UI
   - Scope: frontend
   - Depends on: CH-06
   - Recommended agent: Claude Code (UI/UX routing rules apply)
   - Est. complexity: M
   - Complexity score: Medium · Model class: medium
   - Customer value: HIGH
   - Details: Admin dashboard page: per-model/per-task/per-agent spend over time,
     budget consumption, alert surfacing. Follows CLAUDE.md UI/UX routing steps.

9. CH-08 skill-activation-metrics: precision/recall per skill per model
   - Scope: runtime/matching | metrics
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium · Model class: medium
   - Customer value: MEDIUM
   - Details: Record activation decisions + outcomes in the intent/matching path,
     expose per-skill/per-model precision-recall counters, console surfacing.

10. CH-09 capability-registry-benchmarks: benchmark data in the registry
    - Scope: llm/catalog | data
    - Depends on: NONE
    - Recommended agent: Codex
    - Est. complexity: M
    - Complexity score: Medium · Model class: medium
    - Customer value: MEDIUM
    - Details: Extend `ModelCapabilityRegistry`/catalog with benchmark fields
      (coding/agentic/context benchmarks), static import dataset in-repo with an
      update mechanism, feed router scoring as tiebreaker.
      **A1.3:** per-entry source URL + retrieval date mandatory; dimension
      schema per uar-next-fable.md §2.4; sources: models.dev + provider docs.

11. CH-10 model-comparison-dashboard: side-by-side model comparison UI
    - Scope: frontend
    - Depends on: CH-09
    - Recommended agent: Claude Code (UI/UX routing rules apply)
    - Est. complexity: M
    - Complexity score: Medium · Model class: medium
    - Customer value: MEDIUM
    - Details: Extend `models-page.tsx` catalog with multi-select compare view:
      benchmarks, cost, capabilities side-by-side.

12. CH-11 rag-hardening: decomposition + verification + audit, in-process (D-A)
    - Scope: rag | events
    - Depends on: NONE
    - Recommended agent: Claude Code
    - Est. complexity: L
    - Complexity score: High · Model class: frontier
    - Customer value: MEDIUM
    - Details: Query decomposition (multi-query), retrieval verification pass,
      audit events for retrieval decisions (Rule 34). Keep service extraction
      deferred; design seams so extraction later is mechanical.

### Round 3 — Spec & Distribution (child phase: `spec-v2-distribution`)

13. CH-12 agent-spec-v2: RFC + IR fields
    - Scope: docs/spec | compiler/ir
    - Depends on: CH-04 (dialect), CH-03 (routing semantics to declare against)
    - Recommended agent: Claude Code (spec) + Roo Architect review
    - Est. complexity: M
    - Complexity score: High · Model class: frontier
    - Customer value: MEDIUM
    - Details: RFC v2.0: `model_requirements`, `prompt_dialect`,
      `rag_configuration`, `context_strategy`, `api_harness`; extend
      `AgentDescriptorIR` with backward-compatible parsing (v1.1 still loads).

14. CH-13 compiler-v2-stages: PMPO stages handle v2 fields
    - Scope: compiler
    - Depends on: CH-12
    - Est. complexity: M · Complexity score: Medium · Model class: medium
    - Customer value: MEDIUM
    - Details: s01_frontmatter validation + stage plumbing + descriptor emit/sign
      for the five new sections; round-trip tests.

15. CH-14 conformance-testing: agents run with declared requirements
    - Scope: compiler | runtime | tests
    - Depends on: CH-13
    - Est. complexity: M · Complexity score: Medium · Model class: medium
    - Customer value: MEDIUM
    - Details: Harness validating a compiled descriptor's declared requirements
      (model, dialect, tools, context) are satisfiable at load and honored at run.

16. CH-15 agent-template-library: pre-built agent templates
    - Scope: content | ci
    - Depends on: CH-13
    - Est. complexity: M · Complexity score: Low · Model class: small
    - Customer value: MEDIUM
    - Details: `.agent.md` templates (coding, vision, terminal, data) compiled +
      signed in CI as release artifacts.

17. CH-16 skill-pack-bundling (RESCOPED per A1.4): skill-pack auto-detection + loader upgrades
    - Scope: build | runtime | cli
    - Depends on: CH-13 (only for the s08 pack-version provenance part; loader work is independent)
    - Est. complexity: L · Complexity score: High · Model class: frontier
    - Customer value: HIGH
    - Details: The pack is already bundled (submodule + `builtin_loader.rs`), so
      this change is the fable §6 loader upgrade: detection precedence
      (`UAR_BUILTIN_SKILLS_DIR` → sibling checkout w/ `.claude-plugin/plugin.json`
      → installed plugin locations (highest version) → embedded submodule →
      optional gated fetch), full agentskills.io frontmatter (`metadata`,
      `license`, `compatibility`, `model_routing` → `RouteRequirements`),
      progressive disclosure (lazy body/references load; 279 SKILL.md files),
      nested-skill hierarchy preservation, merge pack `.mcp.json` (7 servers,
      namespaced, opt-in per server), precedence-wins collision policy reusing
      `skill-collision-allowlist.json`, record `(pack_version, source, root)` at
      load + surface in admin UI, s08 records pack version in signed descriptors.
      Skills are NOT compiled into descriptors — agents pin skill name+version.

18. CH-17 eval-targeted-suites: skill-activation / routing / context-efficiency evals
    - Scope: evals
    - Depends on: CH-08 (activation metrics), CH-09 (benchmark registry)
    - Est. complexity: M · Complexity score: Medium · Model class: medium
    - Customer value: MEDIUM
    - Details: Three targeted suites in `evals/` using the existing harness +
      scorers; wire into the two-tier CI gate.

### Round 4 — Integration & Polish (child phase: `integration-and-polish`)

19. CH-18 librefang-a2a-agui-bridge (D-C scope): UAR-side integration surface
    - Scope: api/a2a | api/agui | examples
    - Depends on: CH-01, CH-21
    - Est. complexity: L · Complexity score: High · Model class: frontier
    - Customer value: HIGH
    - Details: A2A task intake contract for external orchestrators (LibreFang),
      AG-UI stream consumption contract + example client, integration doc for the
      LibreFang team. Cross-repo LibreFang work explicitly out of scope.
      **A1.5:** deliverable #1 is the zero-code seam — bossfang `provider_urls`
      → UAR OpenAI-compat endpoint (end-to-end test IS in scope; needs no
      librefang code). librefang already has A2A + pins Prometheus-AGS
      surreal-memory (standardize scope naming); skill bridge exists
      (`librefang-wasm-skill`, `upload-to-bossfang`). Integrate against
      `librefang-api` endpoints — the "50+ page dashboard" does not exist.

19b. CH-21 agui-spec-alignment (NEW per A1.6): official AG-UI event vocabulary
    - Scope: api/sse | frontend adapter | docs
    - Depends on: NONE (can start any time after Round 0)
    - Recommended agent: Claude Code
    - Est. complexity: M
    - Complexity score: Medium · Model class: medium
    - Customer value: HIGH (unlocks CopilotKit / MS Agent Framework / Oracle A2UI interop)
    - Details: Add a spec-conformant AG-UI event stream (RUN_STARTED,
      TEXT_MESSAGE_CONTENT, TOOL_CALL_START/ARGS/END, STATE_DELTA, RUN_FINISHED,
      RUN_ERROR) as a new `stream_mode` (or replace `agui` mode), mapping from
      `NormalizedEvent` in `src/uar/api/sse.rs`; keep legacy `agui.*` names
      behind a compat flag; conformance-check against the AG-UI dojo/client.

20. CH-19 docs-overhaul-deploy-guide: docs match the new architecture
    - Scope: docs
    - Depends on: Rounds 1–3 landed (content exists to document)
    - Est. complexity: M · Complexity score: Low · Model class: small
    - Customer value: MEDIUM
    - Details: Update ARCHITECTURE/README for router+dialect+health, consolidated
      production deployment guide (k8s/helm exist; narrative missing), fix stale
      GKE→AKS CI doc, **document the D-D pin rationale**, document D-B MemPalace
      status. Closes G4.5/G4.6 + rejected-row dispositions.

21. CH-20 perf-security-load (G5 gate): hot-path profile, load test, injection review
    - Scope: all (read-mostly)
    - Depends on: Rounds 1–3
    - Est. complexity: L · Complexity score: High · Model class: frontier
    - Customer value: HIGH
    - Details: Profile router/dialect/context hot path, 1000-concurrent-agent load
      test, prompt-injection resistance review, `server.rs` (4,922 LOC, A1.7) split
      evaluation. Findings feed a release-candidate decision.

### OPERATOR (not a change — cannot be done by an agent)

- **OP-1 Seed the eval baseline:** set `UAR_LLM__API_KEY` secret (+
  `vars.UAR_EVAL_MODEL`), run `eval-nightly` via `workflow_dispatch
  update_baseline=true`, commit `evals/results/starter.baseline.json`, verify a
  deliberate regression fails. Until then the nightly gate fails loudly by design.

## EXECUTION ROUND ORDER

Round 0 (immediate): HK0
Round 1 (parallel, child `foundation-completion`): CH-22 first, then CH-01, CH-02, CH-03, CH-04 (each adds its live case per A2.3)
Round 2 (parallel after CH-04, child `intelligence-completion`): CH-05, CH-06→CH-07, CH-08, CH-09→CH-10, CH-11
Round 3 (child `spec-v2-distribution`, after CH-03/CH-04): CH-12→CH-13→{CH-14, CH-15, CH-16}, CH-17
Round 4 (child `integration-and-polish`): CH-21 → CH-18 (after CH-01+CH-21), CH-19, CH-20

## COMMANDS TO RUN

Round 0 is a direct task (no OpenSpec). Open Round 1 changes now; open later
rounds when their round begins:

/opsx:new proxy-integration-gate
/opsx:new a2a-grpc-enable
/opsx:new postgres-credential-store
/opsx:new provider-health-failover
/opsx:new prompt-dialect-engine

## Sycophancy self-check

- S-02: the "EVERY recommendation" goal conflicts with reality (9 rows already
  done, 1 harmful) — surfaced in Framing; rejected row D-D has explicit rationale.
- S-07: LibreFang integration cut to UAR-side only; RAG extraction deferred;
  MemPalace enablement deferred — three explicit scope cuts.
- S-03: trade-offs surfaced (in-process RAG debt, deferred extraction seam,
  operator-only eval gate, benchmark data staleness risk in CH-09).

PLAN COMPLETE
