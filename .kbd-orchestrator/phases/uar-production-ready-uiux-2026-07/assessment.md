ASSESSMENT: uar-production-ready-uiux-2026-07
Project: universal-agent-runtime (Universal Agent Runtime)
Date: 2026-07-08
Codebase baseline: 18 frontend pages (chat + 17 admin sections) all reachable via routing; Rust backend 387/388 lib tests green. User's added constraint this assess pass: audit every function in the web app for real (not facade) behavior, remove non-essential functionality, and explicitly preserve agent chat/config and provider/model config.
Cross-tool progress: NONE — progress.json shows 0/0 changes, phase just created this session.

IMPLEMENTATION STATUS

**Load-bearing capability 1 — chat + agent configuration (user's explicit "must retain"): DONE, verified solid.**
`chat-page.tsx` → `useChatRuntime` → real `POST /api/chat/completion` SSE stream (confirmed route in `src/server.rs:701`), with real resume (`GET /api/uar/runs/{id}/stream`) and cancel (`POST /api/uar/runs/{id}/cancel`) endpoints, both confirmed registered. `AgentSelector` reads from the real entity graph and posts real agent selection to `POST /api/uar/sessions/{id}/agent-config` (confirmed route `src/server.rs:923`). `agents-page.tsx` (create/edit/delete/AI-builder/memory-settings) all hit real `agents-api.ts` endpoints with optimistic updates and visible error states; `agentLacksModel()` even surfaces a warning icon when an agent has no model configured — directly relevant to the "must retain" constraint. No facades found anywhere in this path.

**Load-bearing capability 2 — provider/model configuration (user's explicit "must retain"): DONE, verified solid.**
`providers-page.tsx` (configure/set-default/remove, per-provider model list) all call real `providers-api.ts` endpoints with optimistic updates and real error surfacing. `AgentEditor` exposes per-agent default + fallback model selection wired to the same policy structure the backend consumes. `models-page.tsx` and `credentials-page.tsx` are substantial (1013 and 363 lines respectively), have no TODO/stub markers, and their mutations trace to real handlers. No facades found.

**Runtime Console — PARTIAL, real dead facades found (agent-verified, `Explore` fork 1):**
- `RuntimeCockpitPage`: Provider Health and Memory Activity side panels are permanently empty — the backend's `to_runtime_entity_event` mapper (`src/uar/api/sse.rs`) never produces these entity types, only test fixtures do. The `RunRow` "Inspect" button has no `onClick` handler at all — a literal dead button.
- `RuntimeRunsPage`: Artifacts panel is permanently empty for the same reason (entity type never emitted).
- `RuntimeApprovalsPage`: fully wired — real `POST /api/uar/runs/{id}/approval` with optimistic update/revert.
- `RuntimeProtocolsPage`: **dead facade for all dynamic content.** AG-UI events, model-route decisions, and A2UI surfaces are never emitted by the backend; the page silently shows an empty state with **no gating/banner** indicating the feature isn't implemented. This is the exact "Protocols page explicit gating" item carried as open debt since `uar-production-readiness-gaps` (2026-06-02) — **confirmed still unresolved**, over a month later. The static "protocol surface" cards (Anthropic REST / OpenAI REST / MCP) are hardcoded labels, not data-driven — harmless but not "wired" to anything.

**Cost Dashboard — PARTIAL, but honest.** Reads real SSE-fed budget-alert data; a code comment explicitly discloses the in-memory/session-scoped limitation rather than implying persistence it doesn't have. No per-agent/per-task budget **configuration** UI exists anywhere in the frontend (confirmed via grep) — matches the CH-06 carryover ("global-only today") first flagged in `uar-next-harness`, **confirmed still open**.

**MCP Health, Tools, Skills, Knowledge, Memory, Compiler — all DONE, fully wired** (agent-verified across both forks). Every CRUD action traces to a real backend route with visible error states. One minor finding: skills UI displays no activation-outcome/success-rate data even though the backend now records it (`record_skill_activation_outcome`, confirmed called in `manager.rs:1620` — the old "CH-08 half-wired" carryover is **actually resolved on the backend**, just never surfaced in the UI. Not a bug, just an unclaimed opportunity).

**Auth — DONE, one minor bug.** `auth-keys-store.ts`'s `revokeKey` swallows errors in an empty `catch {}` — a failed revoke fails silently, inconsistent with every other mutation on this page which surfaces errors.

**Settings — DONE for both flagged historical concerns, re-verified independently:**
- "Config write-back to YAML" (R3 deferred tradeoff, `uar-production-readiness-gaps`): **confirmed still accurate** — settings persist to the DB via `SettingsManager`, never round-trip into `config.yaml`. Disclosed limitation, not a bug, but still true.
- "Agent-config POST error surfacing" (P3 UX polish, carried since `uar-production-readiness-gaps`): **confirmed RESOLVED** — `saveAll()` now rolls back on failure and renders the error via `ErrorBanner` in every panel.

**A2uiTestingPage — functionally fine, but non-essential.** Confirmed real (`GET /api/uar/a2ui/schemas`), but it's a developer/QA testing harness whose form submission is a local-only echo, not a real action. Candidate for removal or dev-only gating per the user's "remove non-essential functionality" instruction.

**About page — DONE**, real `/healthz` check plus static marketing content (appropriate for the page type).

**Credentials admin UI** — the old carryover "SCOPED (prior carryover); open dedicated phase uar-credentials-admin-ui" is **stale/resolved**: `credentials-page.tsx` exists, is fully wired, and needs no dedicated phase.

CROSS-TOOL PROGRESS
- NONE — no cross-tool activity recorded

SPEC GAP SUMMARY (this is the headline finding)

**The project's two most comprehensive CI workflows have never successfully executed a single real test, check, or benchmark, since the initial commit (2026-01-19).**

`.github/workflows/comprehensive-tests.yml` and `.github/workflows/tests-full.yml` both fail at their very first "Pre-flight Checks" / "Checking Prerequisites" step because they require `test-config.yaml` at the repo root — a file that has **never existed** in this project's history (confirmed via `git log --follow --diff-filter=A`, and via `git log -p` showing the `test -f test-config.yaml` check present since the initial commit). Every downstream job — Code Quality, Security Audit, Build Verification, Docker Integration Tests, Comprehensive Tests, Performance Benchmarks — is unconditionally skipped as a result. Confirmed via `gh run list`: **0 successes in the last 30 runs** of `comprehensive-tests.yml`, spanning dozens of Dependabot PRs and merges to `main` over the past 2 days alone; the failure mode is identical going back to the earliest run checked (2026-07-07, a 20-minute run that still failed at the same gate — ruling out a recent regression).

This traces directly to an **abandoned Spec Kit feature**: `specs/001-testing-infrastructure/` describes an ambitious testing-infrastructure system (`TestSuite`/`TestCase`/`CoverageReport`/`QualityGate` entities, certification suites, flaky-test detection, performance-regression tracking) — **0 of 74 tasks in `tasks.md` are checked complete**. The partial dead-code this spec was building (`src/testing/`, ~22.7k lines) was already identified and **deleted** in `eval-harness-hardening`'s HK1 cleanup — but the spec directory, `TESTING.md`'s claims of "100% code coverage," and the two broken CI workflows referencing it were never cleaned up alongside that deletion.

This exact bug was **already found and documented** by a prior tool assessment (`docs/CODEX_ASSESSMENT.md`, 2025-12-31): *"Config file mismatch for tests: `tools/test-all.sh` exports `CONFIG_FILE=test-config.yaml`, but `test-config.yaml` is not an `AppConfig` file... Stop exporting `CONFIG_FILE=test-config.yaml`... use `config.test.yaml` for the server and keep `test-config.yaml` as test-runner config under a separate env var."* That fix was never applied. `docs/CLAUDE_ASSESSMENT.md` also independently lists `test-config.yaml` as a required-but-effectively-fictional config file.

- `.github/workflows/ci.yml` (the workflow that *does* run on every push and actually gates merges) has a pre-existing, unrelated uncommitted working-tree diff — flagged again here since it's now the 2nd consecutive phase to note it, still not resolved.
- No `permissions:` gap or other issue found in the currently-working workflows (`security-audit.yml`, `ci.yml`, `quick-tests.yml`) — this gap is specific to the two "comprehensive" workflows.

BUILD HEALTH
- `cargo test --lib`: 387 passed, 1 ignored, 0 failed — PASS.
- Frontend `vitest run`: 46/46 passed across 12 test files — PASS, but thin: only 12 of 206 non-test `.ts`/`.tsx` source files (~5.8%) have a corresponding unit/component test.
- Frontend `playwright test --list`: 40 e2e tests across 12 spec files parse and enumerate correctly, covering chat (basic, agent selection, no-provider guard, session config), admin (providers, agents, skills, tools, knowledge), and runtime console (visual + event-replay) — solid coverage of the two load-bearing capabilities specifically. **Not executed against a live server this session** (would require booting the full stack; out of scope for a fact-finding assess pass) — their real-world pass/fail status is unverified beyond "the suite parses."
- `comprehensive-tests.yml` / `tests-full.yml`: **FAIL, 0/30 recent successes** — see Spec Gap Summary above. This is the standout build-health finding of this assessment.
- known violations: the `test-config.yaml` gap above; no other build-blocking issues found.

CONSTRAINT CHECK
- AGENTS.md / Prometheus Base Rules Set violations: NONE found in the code inspected this pass.
- Rule 40 (Stop When Done) — worth citing as a cautionary precedent: `specs/001-testing-infrastructure` is a textbook case of scope far exceeding what shipped, left as permanent debt. Useful context for how this phase's own plan should be scoped (don't repeat the pattern).
- Constraint violations: N/A — no `.kbd-orchestrator/constraints.md` exists in this project.

GOAL PROGRESS
1. **Survey and close remaining production-readiness gaps**: PARTIAL — re-verification found 3 old carryover items resolved (credentials UI, agent-config error surfacing, skill-activation-outcome now recorded backend-side) and 2 still genuinely open (CH-06 per-agent budget config, Protocols page gating), **plus one major previously-undiscovered gap**: the comprehensive/full-test CI workflows have never worked, ever.
2. **Audit UI/UX completeness / "ALL functions actually function"**: PARTIAL — 14 of 18 pages verified fully wired with no facades; concrete dead facades found in Runtime Console (Protocols page entirely, 2 Cockpit panels, 1 Runs panel, 1 dead Inspect button); one non-essential page (`A2uiTestingPage`) flagged for removal/gating; one minor error-swallow bug (`revokeKey`). **Both explicitly protected capabilities (agent chat/config, provider/model config) are confirmed genuinely solid, not facades.**
3. **Identify and close test coverage gaps**: NOT MET — the flagship "comprehensive" and "full" test suites have never executed their real jobs in this project's history; frontend unit test coverage is thin (~5.8% file coverage) though what exists passes cleanly; e2e suite is reasonably comprehensive but unverified as actually passing this session.

ASSESSMENT COMPLETE
