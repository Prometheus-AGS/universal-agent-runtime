PLAN: uar-production-ready-uiux-2026-07
Project: universal-agent-runtime (Universal Agent Runtime)
Date: 2026-07-08
OpenSpec available: YES
Changes to implement: 9

CHANGE LIST (ordered)

1. fix-comprehensive-tests-ci-gate: unblock `comprehensive-tests.yml` and `tests-full.yml` at their permanently-failing Pre-flight/Prerequisite check.
   - Scope: ci (2 workflow files), config (new `test-config.yaml`), possibly `tools/test-all.sh`
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: M
   - Complexity score: Medium — requires reconciling two config files' roles, not just creating a file
   - Model class: frontier
   - Customer value: HIGH
   - Details: Per `docs/CODEX_ASSESSMENT.md`'s already-researched fix (2025-12-31, never applied): create a real `test-config.yaml` as test-runner config (coverage thresholds, test-mode settings), stop `tools/test-all.sh` from exporting it as the server's `CONFIG_FILE` (that's `config.test.yaml`'s job), and use a separate env var (e.g. `TEST_CONFIG_FILE`) for the test-runner's own config. Verify by dispatching both workflows for real and confirming they progress past Pre-flight — this is the single highest-value fix in this phase given the headline assessment finding.

2. fix-auth-revoke-key-error-surfacing: stop `auth-keys-store.ts`'s `revokeKey` from silently swallowing errors.
   - Scope: frontend (`auth-keys-store.ts`)
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: S
   - Complexity score: Low — one-line fix, matches the error-surfacing pattern already used elsewhere on the same page
   - Model class: small
   - Customer value: LOW
   - Details: Replace the empty `catch {}` with the same `setError`/toast pattern used by every other mutation on `auth-page.tsx`.

3. upgrade-a2ui-testing-live-round-trip: ~~retire-a2ui-testing-page-from-prod~~ **RESCOPED 2026-07-09** — user rejected removal; kept the page and upgraded it to trigger a real round-trip instead.
   - Scope: backend (`src/uar/a2ui/routes.rs`, new endpoint), frontend (`A2uiTestingPage.tsx` reworked, not deleted)
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: M (revised up from original S — this is now real new feature work, not a removal)
   - Complexity score: Medium — investigation found the real A2UI round-trip already works end-to-end in production chat (`A2uiInputBlock` → real `/artifact-response` endpoint → agent resumes); the actual gap was narrower than assessed: no way to *trigger* that flow on demand for testing. New endpoint (`POST /api/uar/runs/{run_id}/a2ui/test-trigger`) emits a real `ArtifactInputRequest` into an active run; the reworked test page adds an active-run picker and hands off to the real chat UI to complete the round-trip — reuses existing production components entirely, no parallel rendering path.
   - Model class: frontier
   - Customer value: MEDIUM
   - Details: See `openspec/changes/upgrade-a2ui-testing-live-round-trip/design.md` for the full investigation and decision trace. Explicitly does NOT overlap with change #4's (`resolve-runtime-protocols-page-facade`) still-open fix-vs-remove decision — that's about the Runtime Console's read-only dead display panels, a separate concern from this change's interactive trigger-and-complete purpose.

4. resolve-runtime-protocols-page-facade: close the "Protocols page gating" carryover (open since `uar-production-readiness-gaps`, 2026-06-02).
   - Scope: frontend (`runtime-console-page.tsx`'s `RuntimeProtocolsPage`), possibly backend (`src/uar/api/sse.rs`) if wiring real data
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: M (gating banner) or L (real backend wiring) — **product decision needed at execute time**
   - Complexity score: Medium — the cheap fix (explicit "not yet implemented" banner) is well-scoped; the expensive fix (real AG-UI/model-route/A2UI event emission from the backend) is a genuinely new cross-cutting feature, not a bug fix
   - Model class: frontier
   - Customer value: MEDIUM
   - Details: **Flag to the user before implementing**: do we (a) add an honest "not yet implemented" gating banner (cheap, matches this project's own established practice of disclosing rather than hiding gaps), or (b) actually build backend emission for AG-UI events / model-route decisions / A2UI surfaces so the page becomes real (a genuinely new feature, out of proportion with "fix a facade")? Default recommendation: (a) now, defer (b) to a dedicated future phase if wanted.

5. resolve-runtime-cockpit-dead-panels: close the Provider Health + Memory Activity permanently-empty panels on `RuntimeCockpitPage`.
   - Scope: frontend (`runtime-console-page.tsx`), possibly backend (`src/uar/api/sse.rs`)
   - Depends on: NONE (parallel with #4, same fix-vs-remove tension, same file — sequence after #4 if both touch `runtime-console-page.tsx` to avoid merge friction, or combine into one change if the plan is later revised)
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: S (remove) or M (wire real backend health/memory-event emission)
   - Complexity score: Medium
   - Model class: medium
   - Customer value: LOW
   - Details: Same fix-vs-remove decision as #4. Provider health specifically may be low-hanging fruit since a `/api/uar/mcp/health`-style real health signal may already exist server-side for other purposes (confirmed `McpHealthPage` has one) — worth checking whether Provider Health can reuse that instead of inventing new backend plumbing.

6. resolve-runs-artifacts-and-inspect-button: fix the two remaining Runtime Console dead-facade items — the permanently-empty Artifacts panel on `RuntimeRunsPage`, and the `RunRow` "Inspect" button with no `onClick` at all.
   - Scope: frontend (`runtime-console-page.tsx`)
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: S
   - Complexity score: Low — the Inspect button just needs to open a detail view of data the page already has; Artifacts panel follows #4/#5's fix-or-remove pattern
   - Model class: small
   - Customer value: MEDIUM
   - Details: The Inspect button is the cheapest, highest-visibility fix in this whole cluster — a literally-dead button is worse UX than a removed feature. Wire it to open/expand the existing run detail data already loaded into the graph.

7. bdd-chat-scenario-suite: build a Cucumber/Gherkin + Playwright BDD suite proving every major chat use case, with video evidence per scenario.
   - Scope: new test infrastructure (likely `tests/bdd/` or similar, per the `bdd-testing` skill's convention), frontend (none, test-only)
   - Depends on: NONE (chat + agent/provider config already confirmed solid at assessment time — no need to wait on Rounds 1-2)
   - Recommended agent: Claude Code (self-executing), using the `bdd-testing` skill to scaffold and the `bdd-video-proof` skill to capture + IPFS-pin evidence
   - Est. complexity: L
   - Complexity score: High — new testing framework integration (Cucumber.js, not just more Playwright specs), scenario design across a genuine feature matrix, video capture pipeline
   - Model class: frontier
   - Customer value: HIGH (this is the user's explicit, detailed ask)
   - Details: Per the user's explicit instruction, cover at minimum: (a) chat with no knowledge base attached, (b) chat with a knowledge base enabled and memory/context retrieval actually influencing a response, (c) chat with skills activated (a skill visibly selected/applied mid-conversation), (d) chat with tool calls (a tool invoked and its result surfaced in the transcript), (e) agent selection/switching mid-session, (f) provider/model configuration affecting which model actually answers. Each scenario gets a Gherkin `.feature` file (readable scenario documentation, doubling as "the scenarios we support" record) and a video-proof capture. Record the full scenario list in a checked-in registry (e.g. `docs/BDD_SCENARIOS.md`) so support claims are documented, not just passively tested.

8. bootstrap-docusaurus-site: scaffold a real Docusaurus site ingesting the existing `docs/*.md` content.
   - Scope: new project (likely `website/` or `docs-site/` directory), doc content migration
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: L
   - Complexity score: High — new framework, new build/deploy surface, content migration and information-architecture decisions (nav structure, versioning, search) for ~20+ existing docs/*.md files
   - Model class: frontier
   - Customer value: MEDIUM
   - Details: User confirmed via `AskUserQuestion` (2026-07-08): bootstrap a new Docusaurus site from the existing `docs/` markdown, styled with the Prometheus/`travisjames.ai` branding tokens (`.claude/skills/prometheus-entity-skills/_shared/references/branding.md` — 4-font system, ember-orange primary, light/dark HSL tokens). Needs a decision on where it's hosted/deployed (not decided yet — flag to user at execute time) and whether it's a new pnpm/npm workspace member or fully standalone.

9. refresh-readme-diagrams-and-branding: fix README.md's stale claims and apply branding.
   - Scope: docs (`README.md`, 683 lines, 2 mermaid diagrams)
   - Depends on: NONE
   - Recommended agent: Claude Code (self-executing)
   - Est. complexity: M
   - Complexity score: Medium — not just a diagram touch-up; a real factual correction plus a full branding pass
   - Model class: medium
   - Customer value: MEDIUM
   - Details: **Concrete factual error found and confirmed this session**: README.md's "High-Level Goals" #6 claims *"HTML-centric UI — HTMX, Web Components, Alpine.js; no React, Next.js, or SPA routers"* — this directly contradicts the actual codebase, which is a 100%-React/TypeScript SPA (`react-router-dom` `BrowserRouter`, TanStack Query, Zustand, shadcn-ui, assistant-ui — confirmed extensively during this phase's assessment, zero HTMX/Alpine/Web-Components anywhere in `frontend/src`). The 2nd mermaid diagram (realtime data-flow) checked out as accurate; the 1st (architecture overview) is structurally close but should be re-verified end-to-end against current code, not just this one claim. **Note**: `CLAUDE.md` opens with the identical stale claim ("HTML-first frontend technologies (HTMX, Web Components, Alpine.js)") — out of the user's explicit ask (README only) but flagged as a closely-related fix worth folding in or doing as an immediate follow-up.

EXECUTION ROUND ORDER
Round 1 (parallel, no shared files): `fix-comprehensive-tests-ci-gate`, `fix-auth-revoke-key-error-surfacing`, `upgrade-a2ui-testing-live-round-trip`
Round 2 (parallel-ish, all touch `runtime-console-page.tsx` — sequence to avoid merge conflicts, or combine at execute time): `resolve-runtime-protocols-page-facade`, `resolve-runtime-cockpit-dead-panels`, `resolve-runs-artifacts-and-inspect-button`
Round 3 (independent, new scope): `bdd-chat-scenario-suite`
Round 4 (independent, docs/branding): `bootstrap-docusaurus-site`, `refresh-readme-diagrams-and-branding`

Rounds 3 and 4 have no dependency on Rounds 1–2 and could run in parallel with them if desired — the ordering above is by priority (assessment-driven fixes first), not by hard dependency.

**Scope note**: this plan has grown to 9 changes spanning bug fixes, a product-facing feature-completeness decision (Runtime Console), a new testing framework integration, and a new documentation platform. `goals.md`'s own "Expected shape" section anticipated this and suggested nested child phases (`/kbd-new-child`) if assessment revealed separable sub-initiatives — it did. Recommend considering splitting Round 3 (BDD suite) and Round 4 (docs/Docusaurus) into their own child phases at execute time so each track gets its own focused reflection, rather than one flat reflection trying to cover bug fixes, a new test framework, and a new docs platform at once. Not decided here — flagging for the user's call before `/kbd-execute`.

COMMANDS TO RUN
/opsx:new fix-comprehensive-tests-ci-gate
/opsx:new fix-auth-revoke-key-error-surfacing
/opsx:new upgrade-a2ui-testing-live-round-trip
/opsx:new resolve-runtime-protocols-page-facade
/opsx:new resolve-runtime-cockpit-dead-panels
/opsx:new resolve-runs-artifacts-and-inspect-button
/opsx:new bdd-chat-scenario-suite
/opsx:new bootstrap-docusaurus-site
/opsx:new refresh-readme-diagrams-and-branding

PLAN COMPLETE
