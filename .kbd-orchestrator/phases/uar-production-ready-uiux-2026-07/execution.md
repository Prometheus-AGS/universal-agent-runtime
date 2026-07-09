EXECUTION: uar-production-ready-uiux-2026-07
Project: universal-agent-runtime (Universal Agent Runtime)
Date: 2026-07-08
Selected backend: openspec
Dispatched to: SELF (Claude Code CLI, self-executing via /kbd-apply)
Backend rationale: OpenSpec directory exists at project root and is this project's established backend for every prior phase. User confirmed (2026-07-08, AskUserQuestion) to keep this as a single flat phase rather than splitting the BDD/Docusaurus tracks into child phases — all 9 changes execute under this one phase.
Backend entrypoint: /kbd-apply <change-id>, one change at a time, per this project's standing rule to never invoke bare /opsx:apply.
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-production-ready-uiux-2026-07/plan.md

EXECUTION SCOPE

- fix-comprehensive-tests-ci-gate: fix the test-config.yaml gap so comprehensive-tests.yml/tests-full.yml actually run.
- fix-auth-revoke-key-error-surfacing: stop revokeKey from silently swallowing errors.
- retire-a2ui-testing-page-from-prod: remove/gate the dev-only A2UI testing page.
- resolve-runtime-protocols-page-facade: close the Protocols page gating carryover.
- resolve-runtime-cockpit-dead-panels: close Cockpit's Provider Health + Memory Activity dead panels.
- resolve-runs-artifacts-and-inspect-button: fix Runs page Artifacts panel + dead Inspect button.
- bdd-chat-scenario-suite: Cucumber/Gherkin + Playwright BDD suite with video-proof evidence for all chat use cases.
- bootstrap-docusaurus-site: new Docusaurus site ingesting docs/.
- refresh-readme-diagrams-and-branding: fix README's stale HTMX/Alpine claim + apply branding.

DISPATCH CONTRACTS

- fix-comprehensive-tests-ci-gate → SELF (claude-code)
  Entry: /opsx:new fix-comprehensive-tests-ci-gate, then /kbd-apply fix-comprehensive-tests-ci-gate
  Model class: frontier | Concrete model: claude-sonnet-5 (session default, no model_policy override in project.json)
  Model rationale: reconciling test-config.yaml vs config.test.yaml's roles per docs/CODEX_ASSESSMENT.md's proposed fix is a real design decision, not mechanical.
  Progress file: .kbd-orchestrator/phases/uar-production-ready-uiux-2026-07/progress.json

- fix-auth-revoke-key-error-surfacing → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: small. Mechanical, matches an existing pattern on the same page.

- retire-a2ui-testing-page-from-prod → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: small. Routing/nav removal, no new logic.

- resolve-runtime-protocols-page-facade → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: frontier.
  BLOCKING PRODUCT DECISION: gating banner vs. real backend event emission — must be resolved with the user via AskUserQuestion before implementation, per plan.md's explicit flag. Plan's default recommendation is the gating banner.

- resolve-runtime-cockpit-dead-panels → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: medium.
  Same fix-vs-remove decision as above; sequence after resolve-runtime-protocols-page-facade since both touch runtime-console-page.tsx.

- resolve-runs-artifacts-and-inspect-button → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: small.
  Sequence after the two changes above (same file).

- bdd-chat-scenario-suite → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply, using the bdd-testing skill to scaffold Cucumber/Gherkin + Playwright scenarios and the bdd-video-proof skill to capture + IPFS-pin video evidence per scenario.
  Model class: frontier | Customer value: HIGH (user's explicit, detailed ask).
  Independent of Rounds 1–2 — chat + agent/provider config already confirmed solid at assessment time.

- bootstrap-docusaurus-site → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: frontier.
  User confirmed scope via AskUserQuestion (2026-07-08): bootstrap new, styled with .claude/skills/prometheus-entity-skills/_shared/references/branding.md tokens. Hosting/deployment target and workspace-member-vs-standalone are open questions to surface to the user during this change's design step.

- refresh-readme-diagrams-and-branding → SELF (claude-code)
  Entry: /opsx:new + /kbd-apply. Model class: medium.
  Concrete factual error already confirmed (HTMX/Alpine claim vs. actual React/TypeScript SPA) — no further investigation needed before fixing.

APPROVAL GATES

- git push origin main — confirm with the user before pushing, per this project's standing approval gate (every prior phase this session required explicit confirmation before push).
- resolve-runtime-protocols-page-facade and resolve-runtime-cockpit-dead-panels: fix-vs-remove product decisions must be surfaced to the user before implementation, not defaulted silently.
- bootstrap-docusaurus-site: hosting/deployment destination is undecided — surface to the user before assuming a specific host (GitHub Pages, Vercel, etc.) or CI deploy workflow.

FALLBACK CONDITIONS

- If any Round 2 change's scope balloons into "wire real backend emission" (the expensive option), consider splitting that specific change into its own dedicated future phase rather than absorbing unplanned backend feature work into this phase — matches this project's own documented lesson about scope creep (`specs/001-testing-infrastructure`).

VERIFICATION REQUIREMENTS

- fix-comprehensive-tests-ci-gate: dispatch both workflows for real via `gh workflow run` and confirm they progress past Pre-flight (matches this project's established "CI Trigger Actually Fires" requirement — no claim of "fixed" without an observed real run).
- Runtime Console changes (Round 2): `pnpm run build` + relevant Playwright e2e specs (`runtime-console-visual.spec.ts`, `runtime-event-replay.spec.ts`) still pass.
- bdd-chat-scenario-suite: each scenario's video-proof artifact actually exists and is IPFS-pinned; the scenario registry doc lists every scenario with a passing status.
- bootstrap-docusaurus-site: `npm run build` (or pnpm equivalent) for the new site succeeds locally.
- refresh-readme-diagrams-and-branding: no remaining HTMX/Alpine/Web-Components claims anywhere in README.md; mermaid diagrams render (validate via a mermaid CLI or visual check).

PROGRESS LEDGER

- [PENDING] fix-comprehensive-tests-ci-gate — claude-code
- [PENDING] fix-auth-revoke-key-error-surfacing — claude-code
- [PENDING] retire-a2ui-testing-page-from-prod — claude-code
- [PENDING] resolve-runtime-protocols-page-facade — claude-code
- [PENDING] resolve-runtime-cockpit-dead-panels — claude-code
- [PENDING] resolve-runs-artifacts-and-inspect-button — claude-code
- [PENDING] bdd-chat-scenario-suite — claude-code
- [PENDING] bootstrap-docusaurus-site — claude-code
- [PENDING] refresh-readme-diagrams-and-branding — claude-code

OUTPUTS

- NONE yet — execute phase just dispatched

BLOCKERS

- 2 fix-vs-remove product decisions (Round 2) — not yet resolved with the user.
- Docusaurus hosting/deployment target — not yet resolved with the user.

REFLECTION HANDOFF

- Whether the Round 2 fix-vs-remove decisions landed on the cheap or expensive option, and whether that scope stayed contained.
- Whether the BDD suite's video-proof pipeline worked end-to-end (IPFS pinning is an external dependency not previously exercised in this project).
- Whether keeping this as one flat phase (vs. child phases) made the eventual reflection harder to write clearly, given the 4 distinct tracks — useful data point for the next time a phase grows this broad.

EXECUTION READY
