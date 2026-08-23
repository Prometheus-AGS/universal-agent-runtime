ASSESSMENT: uar-branded-documentation-site
Project: universal-agent-runtime
Date: 2026-08-22
Codebase baseline: Commit 9274dc2d contains a Docusaurus 3.10.2/React 19 site, 38 tracked README files, 373 tracked documentation paths, 99 OpenSpec capability directories, and a 546-file checked-in .prometheus history, but the public Pages endpoint currently serves only generated TypeScript SDK documentation.
Cross-tool progress: none for this phase; the completion evidence visible in progress.json belongs to the completed prior phase and does not verify this documentation phase.

IMPLEMENTATION STATUS
- Documentation estate and ownership inventory: PARTIAL — the repository has broad narrative, API, historical, certification, design, SDK, tool, test, and vendored documentation, but no authoritative manifest classifies public current docs, public historical docs, private-synthesis-only history, generated mirrors, or excluded vendor material. Of 38 README files, 31 are directly UAR-owned surfaces, 5 are iterative-evolver mirrors, and 2 are vendored.
- README consistency: PARTIAL — the root README describes the current React/server-full architecture and links to the intended docs URL, but the readme-presentation spec's hero and badge row are absent, the public link resolves to 404 below the site root, and subordinate README files still contain stock Docusaurus instructions, placeholder cookbooks, and superseded guidance.
- Docusaurus portal foundation: PARTIAL — website/ already pins Docusaurus 3.10.2 and React 19, enables Mermaid, fails on broken links, declares GitHub Pages URL/baseUrl settings, and contains 23 narrative Markdown pages. It lacks an effective complete portal publication, deterministic search, a public-content sanitizer, and a docs-specific immutable non-root container.
- Docusaurus brand parity: STUB — website/src/css/custom.css is the default green Infima palette; the home page still says “Docusaurus Tutorial - 5min,” uses placeholder metadata, stock feature cards, stock social art, and a Docusaurus crocodile logo. It does not consume the shipped app's Space Grotesk/Geist/JetBrains Mono typography, ember/cyan palette, four-step surface ladder, Flat 2.0 treatment, UAR wordmark, responsive shell language, or high-contrast behavior.
- Product and theory documentation: PARTIAL — the root README, architecture guide, support matrix, configuration, API reference, SDK, skills, security, deployment, and protocol pages cover important current behavior. The site has only 23 narrative pages against 99 capability directories and 22 catalogued product surfaces; compiler has no site coverage and knowledge, memory, governance, observability, metrics, cost, tenancy, approvals, credentials, realtime, shutdown, embedded profiles, and provider/model behavior are thin or fragmented.
- Architecture and design decision history: PARTIAL — 17 numbered ADRs plus KBD goals, assessments, plans, executions, reflections, and decision logs exist, but they are not presented as one traceable history. Older assessment and design files make superseded AGPL, HTMX, purple-theme, CI-coverage, placeholder, and unimplemented claims without a consistent historical banner.
- Karpathy/.prometheus synthesis: MISSING — .prometheus contains 546 tracked files (536 wiki records, decisions, session log, gotchas, events, and a postmortem) totaling about 2.0 MB, but no public synthesis exists. Raw publication is unsafe: 468 files contain machine-local absolute paths and 74 contain credential-related terms; raw events and session records must remain private synthesis inputs.
- Testing methodology history: MISSING — current site content has no occurrence of “negative control” or “real model” and does not explain the evolution from unit/coverage and synthetic/recorded checks to local, bounded, genuine-model functional integration. Historical coverage ADRs remain accepted even though the operating policy now forbids routine GitHub Actions testing and rejects synthetic inference as readiness evidence.
- GitHub Pages deployment: PARTIAL — .github/workflows/docs.yml is deployment-oriented and uses upload-pages-artifact/deploy-pages, but its latest run failed at “pnpm copy:adr” because the workflow installed npm dependencies without installing pnpm. A second workflow, typescript-sdk-docs.yml, deploys to the same Pages environment and most recently succeeded, replacing the portal with TypeScript API documentation.
- Public site and repository metadata: MISSING — GitHub Pages is enabled and HTTPS-enforced, but /docs/intro, /docs/intro/, /docs/intro.html, and /docs/ all return 404. The site root returns 200 with title “@prometheus-ags/universal-agent-runtime-sdk.” GitHub repository description and homepageUrl are both empty.
- Documentation validation: PARTIAL — Docusaurus broken-link failure and a local documentation-truth validator exist, but the truth gate checks only 11 canonical files. The Vale wrapper exits successfully when Vale is absent. No validator enforces source classification, current-vs-historical banners, .prometheus sanitization, complete README coverage, site-route coverage, or the single Pages publisher invariant.
- Generated API references: PARTIAL — navbar links and workflow staging exist for Rust and TypeScript; the failed portal workflow prevents them from being published as one site. Python Sphinx is described as published but is not staged by docs.yml, while the standalone TypeScript publisher competes with the portal.

CROSS-TOOL PROGRESS
- NONE — progress.json records zero changes and zero implementation tasks for uar-branded-documentation-site. Its evidence/certification/publication fields are inherited runtime-level state from the prior phase and are not evidence for this phase.

SPEC GAP SUMMARY
- The existing dev-portal-2026 spec describes an earlier minimum portal and still requires Vale to run in the GitHub Pages workflow. That conflicts with the current deployment-only Actions policy; prose validation must run locally while the workflow only produces and deploys the already locally validated site contract.
- The earlier docs-hosted-rustdoc-typedoc-docusaurus-ia change is fully checked off but remains unarchived, explicitly leaves full content and end-to-end API generation for follow-up, and does not represent the newly observed broken deployment or publisher collision.
- documentation-truth-gate covers only 11 files and a narrow prohibited-pattern list; it does not govern all 38 README files, 373 documentation paths, or public/private source classification.
- readme-presentation requires a branded hero and badges that the root README does not contain.
- No current spec defines the app-matched Docusaurus brand contract, public synthesis/provenance rules for .prometheus, testing-methodology history, single Pages publisher, repository homepage metadata, or deployed-route validation.
- Historical records cannot simply be rewritten as present-tense truth. The plan must preserve them with dated supersession banners and link them to current authority, while canonical guides are corrected in place.
- Public docs must synthesize .prometheus evidence rather than copy raw logs. Machine paths, possible credential context, private operational details, and raw conversations are exclusion inputs, not publishable content.

BUILD HEALTH
- build check: UNKNOWN — no local build or test was run during Assess because implementation has not started and project rules defer verification until its tier.
- latest Docusaurus deployment: FAIL — GitHub run 32593095935 exited 127 with “pnpm: not found” after npm ci invoked a build script containing pnpm copy:adr.
- deployed portal route check: FAIL — the configured Docusaurus document routes return HTTP 404; the Pages root is the TypeScript SDK artifact.
- known violations: stock/unbranded site assets; broken portal deployment; two competing Pages publishers; empty GitHub homepage metadata; incomplete docs coverage; fail-open prose lint wrapper; no publication sanitizer; stale unmarked historical claims.
- test coverage: NONE — no new phase implementation exists, so no phase verification applies yet.

CONSTRAINT CHECK
- AGENTS.md violations: NONE introduced by this assessment. Both docs workflows are deployment publishers, but the plan must keep prose linting, link validation, accessibility checks, and other routine verification local.
- constraints.md violations: N/A — no constraints.md exists.
- publication safety: raw .prometheus content must not be copied to the public site; only reviewed, attributed synthesis is compatible with the phase goal and branded-site skill.

GOAL PROGRESS
- Rewrite every README and documentation surface: PARTIAL — extensive current docs exist, but ownership classification, complete audit, stale-history banners, subordinate README repair, and consistent navigation are missing.
- Branded Docusaurus matching the React 19 app: NOT MET — an unbranded stock site exists.
- Explain runtime theory, purpose, architecture, profiles, protocols, APIs, SDKs, tools, skills, knowledge, tenancy, security, operations, and boundaries: PARTIAL — core guides exist, but coverage is fragmented and several shipped surfaces are absent or thin.
- Turn .prometheus history into traceable architecture/design documentation: NOT MET — sources exist but no safe synthesis or provenance map exists.
- Document testing-methodology history: NOT MET — current site omits the required evolution, failures, limits, negative controls, and real-model standard.
- Add a deployment-only GitHub Pages workflow: PARTIAL — a compliant deployment shape exists, but it fails and competes with a second publisher.
- Validate deployment and expose its URL in README/navigation/repository homepage: PARTIAL — the README URL exists, but the portal routes fail and homepageUrl is empty.
- Audit claims against main, versions.toml, OpenSpec, and KBD history: PARTIAL — isolated truth gates and support artifacts exist, but there is no estate-wide reconciliation.

ASSESSMENT COMPLETE
