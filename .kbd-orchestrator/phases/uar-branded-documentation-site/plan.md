PLAN: uar-branded-documentation-site
Project: universal-agent-runtime
Date: 2026-08-22
OpenSpec available: YES
Changes to implement: 11

PLANNING BASIS
- Baseline: `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` on current `main`.
- Existing foundation: `website/` already contains Docusaurus 3.10.2, React 19,
  Mermaid support, a pinned `package-lock.json`, and 23 narrative pages. This phase
  updates that site; it does not scaffold another portal.
- Documentation estate: 38 tracked README files, 393 tracked paths under `docs/`
  and `website/`, 99 OpenSpec capability directories, 21 inventoried product
  surfaces, 17 numbered
  ADRs, and 546 tracked `.prometheus` files.
- Publication defect: the live Pages root currently serves the generated TypeScript
  SDK artifact, the intended Docusaurus routes return 404, `docs.yml` fails because
  an npm install invokes `pnpm copy:adr`, and `typescript-sdk-docs.yml` competes for
  the same Pages environment.
- Privacy boundary: raw `.prometheus` logs are evidence inputs, not public content.
  They contain machine-local paths and credential-related context. Only reviewed,
  attributed synthesis may enter the public site.
- Verification order: implementation changes 1–10 reach code/content completion
  before change 11 runs the local functional documentation gate. GitHub Actions
  performs deployment execution and deployment validation only.

CHANGE LIST (ordered)

1. establish-documentation-publication-contract: Define the public documentation source, truth, and publication contract.
   - Scope: OpenSpec | documentation manifests | local validation tooling
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Classify every documentation source as `public`,
     `public-normalize`, `private-synthesis-only`, or `excluded`; define the
     canonical site IA, required route inventory, provenance format, current versus
     historical treatment, README ownership, and the single-Pages-publisher
     invariant. Amend `customer-documentation`, `dev-portal-2026`,
     `documentation-truth-gate`, and `readme-presentation` so local verification—
     not GitHub Actions—owns prose, link, accessibility, and truth gates. Reconcile
     and supersede the completed-but-unarchived
     `docs-hosted-rustdoc-typedoc-docusaurus-ia` change instead of duplicating its
     earlier minimum-portal claims.

2. repair-single-pages-portal: Restore one deterministic Docusaurus publication path.
   - Scope: website package contract | generated API staging | GitHub Pages workflow | workflow policy validator
   - Depends on: establish-documentation-publication-contract
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Make `website/package-lock.json` and npm the one package-manager
     contract for the site, including replacing the build-time `pnpm copy:adr`
     mismatch. Consolidate Rust and TypeScript API-reference staging beneath the
     portal, remove the competing standalone TypeScript Pages publisher, and make
     `.github/workflows/docs.yml` the only Pages deployer. The workflow may install,
     build, package, deploy, and validate the deployed artifact; it must not run
     unit, integration, lint, conformance, accessibility, or other routine tests.

3. brand-uar-docusaurus-site: Make the portal visibly and behaviorally match the shipped React 19 application.
   - Scope: Docusaurus theme | CSS tokens | brand assets | homepage | navigation | search
   - Depends on: repair-single-pages-portal
   - Recommended agent: Codex with the build-branded-docusaurus and required UI/UX routing skills
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Replace stock Docusaurus copy, crocodile assets, green Infima theme,
     placeholder social card, and tutorial homepage with the UAR wordmark, runtime
     purpose, app typography, ember/cyan palette, four-step surface ladder, and Flat
     2.0 interaction language. Preserve visible keyboard focus, responsive behavior,
     dark/light/high-contrast readability, coordinated Mermaid themes, and add
     deterministic local search. Prefer stable theme classes and CSS; swizzle only
     where the required structure cannot be expressed safely otherwise.

4. document-uar-theory-and-architecture: Explain why UAR exists and how its execution model works.
   - Scope: website concepts | architecture guides | diagrams | glossary
   - Depends on: establish-documentation-publication-contract
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Write the public conceptual spine: problem statement, runtime theory,
     agent/host trust boundary, turn and execution lifecycle, capability inversion,
     profiles, protocol boundaries, event flow, persistence, delegation, and the
     relationship between server-full, minimal, and embedded-mobile. Ground every
     present-tense claim in current source, the operator-owned `versions.toml` when
     present, canonical OpenSpec, or observed product behavior and mark profile
     limits explicitly. `versions.toml` is absent from the planning checkout and
     cannot be cited as inspected evidence.

5. document-inference-skills-knowledge-and-agents: Document the primary product workflows end to end.
   - Scope: website user guides | provider/model configuration | agents | skills | knowledge | memory
   - Depends on: establish-documentation-publication-contract
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Cover provider and model setup, inference, agent creation, built-in and
     configured skills, scoped activation/governance behavior, knowledge ingestion
     and retrieval, memory boundaries, and the packaged UI/API paths. Describe only
     supported behaviors and distinguish genuine model-backed paths from examples,
     fixtures, or non-certifying diagnostics.

6. document-security-tenancy-governance-and-operations: Publish the operating and isolation contract.
   - Scope: website security | tenancy | governance | credentials | approvals | observability | operations
   - Depends on: establish-documentation-publication-contract
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Document authentication, JWT/JWKS behavior, RustCrypto choice, tenant
     isolation, Cedar/governance limits, credentials, approvals, auditability,
     metrics, costs, logs, realtime/SSE behavior, graceful shutdown, recovery, and
     embedded/offline boundaries. State fail-closed conditions and profile-specific
     exclusions without converting prior release evidence into broader claims.

7. document-apis-sdks-tools-and-deployment: Complete developer and operator reference coverage.
   - Scope: website reference | API navigation | Rust/Python/TypeScript SDKs | tools | protocols | deployment
   - Depends on: establish-documentation-publication-contract
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Reconcile supported HTTP, realtime, AG-UI, A2A, MCP, A2UI, CLI, native
     tool, and SDK surfaces with generated references and current source. Document
     local/server deployment, packaging, platform limits, configuration authority,
     and versioned support boundaries. Remove or qualify the existing Python API
     publication claim unless an actual staged artifact supports it.

8. reconcile-uar-readme-estate: Make every README and retained documentation surface point to current authority.
   - Scope: root README | 31 UAR-owned READMEs | generated README sources | historical banners | navigation
   - Depends on: document-uar-theory-and-architecture, document-inference-skills-knowledge-and-agents, document-security-tenancy-governance-and-operations, document-apis-sdks-tools-and-deployment
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Refresh the root hero, tagline, existing logo use, badges, diagrams,
     quickstart, support boundaries, and portal link; then reconcile every UAR-owned
     subordinate README against the canonical portal. Update the source of the five
     iterative-evolver mirror READMEs and regenerate them rather than editing mirrors
     independently. Audit the two vendored READMEs but do not rewrite third-party
     content; record them as excluded. Retained historical documents receive dated
     supersession banners and links to current authority rather than rewritten history.

9. publish-uar-architecture-decision-history: Turn the observed project history into a traceable public architecture narrative.
   - Scope: public history guides | ADR index | KBD/OpenSpec provenance | architecture timeline
   - Depends on: establish-documentation-publication-contract, document-uar-theory-and-architecture
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: MEDIUM
   - Details: Review the complete checked-in `.prometheus` history, KBD phase
     artifacts, OpenSpec changes, ADRs, decisions, gotchas, and postmortems; synthesize
     the decisions, reversals, and present authority with source references. Preserve
     uncomfortable corrections—AGPL, HTMX, purple-theme, placeholder, CI-testing,
     and other superseded claims—without inventing motives. The publication
     sanitizer must reject raw logs, conversations, machine paths, secrets, and
     unreviewed wiki copies.

10. publish-uar-testing-methodology-history: Document how UAR evidence standards changed and what they do not prove.
    - Scope: testing history | evidence taxonomy | negative controls | local verification policy
    - Depends on: establish-documentation-publication-contract
    - Recommended agent: Codex
    - Est. complexity: M
    - Customer value: MEDIUM
    - Details: Explain the progression from coverage-oriented unit checks and
      synthetic/recorded providers to local, bounded, real-model functional
      integration; include failed soak attempts, why elapsed synthetic runtime was
      rejected, negative-control requirements, and per-profile evidence limits.
      Historical methods remain dated evidence, not silently rewritten compliance.
      GitHub Actions is documented as deployment-only.

11. certify-and-publish-uar-docs: Run the completed portal’s local functional gate, deploy it, and expose the validated URL.
    - Scope: local docs verification | Pages deployment validation | repository metadata | final evidence
    - Depends on: brand-uar-docusaurus-site, reconcile-uar-readme-estate, publish-uar-architecture-decision-history, publish-uar-testing-methodology-history
    - Recommended agent: Codex for local verification; Manual approval only for any repository setting not already authorized
    - Est. complexity: M
    - Customer value: HIGH
    - Details: After changes 1–10 are code/content complete, run the frozen local
      install and production build, source-classification/sanitizer gate, truth and
      coverage checks, broken-link and representative-route checks, generated API
      staging checks, responsive light/dark screenshots, keyboard/accessibility
      checks, and the OpenSpec strict validations. Then publish through the sole
      deployment workflow, validate the live root and representative deep routes,
      and set the GitHub repository homepage field to the observed working URL. Record
      commands, outputs, source SHA, limits, and any unverified claims; do not turn a
      documentation result into a runtime-readiness verdict.

EXECUTION ROUND ORDER
Round 1 (serial foundation): establish-documentation-publication-contract
Round 2 (serial build surface): repair-single-pages-portal, then brand-uar-docusaurus-site
Round 3 (parallel, isolated content roots): document-uar-theory-and-architecture; document-inference-skills-knowledge-and-agents; document-security-tenancy-governance-and-operations; document-apis-sdks-tools-and-deployment; publish-uar-testing-methodology-history
Round 4 (after canonical guides): reconcile-uar-readme-estate and publish-uar-architecture-decision-history
Round 5 (only after all implementation/content is complete): certify-and-publish-uar-docs

EXECUTION DISCIPLINE
- Create one worktree per change under `~/.claude/worktrees/`; commit per change.
- Parallel Round 3 changes own separate documentation category directories. The
  publication contract owns the route manifest, and the branding change owns shared
  theme/navigation files, preventing concurrent edits to the same files.
- Rebase each completed change on the preceding accepted baseline; do not merge
  sideways between worktrees.
- Do not push a partial phase. The repaired deployment workflow must not publish a
  stock, incomplete, or partially reconciled portal.
- Keep `.prometheus` version-controlled and append-only. Public pages cite reviewed
  synthesis; they never embed raw `.prometheus` records.
- Implementation checks stay local. GitHub Actions is limited to Pages deployment
  execution and deployed-artifact validation.
- All present-tense support claims name their profile and source. Historical records
  are preserved with dated supersession metadata.

EXPLICIT TRADE-OFFS AND SCOPE CUTS
- A docs-specific immutable container is deferred. The branded-site skill recommends
  one, but this phase’s requested delivery target is GitHub Pages and the repository
  already has separate runtime-container concerns. Adding a second delivery target
  would not unblock or validate the requested public site.
- “Rewrite all documentation” does not mean falsifying or deleting history. Current
  UAR-owned docs are corrected; historical artifacts are retained with banners;
  generated mirrors are changed at their source; vendored docs are audited and
  excluded from semantic rewriting.
- Raw Karpathy/session logs are not publishable. Completeness means every record is
  considered through the classification/provenance process, not copied to the web.
- Deterministic local search remains in scope because complete product documentation
  is not practically navigable without discovery; hosted search and analytics are
  deferred because neither is required for Pages delivery.

PHASE EXIT CRITERIA
- All 38 README files have an explicit disposition; every UAR-owned current README is
  reconciled, generated mirrors match their source, and vendored files remain intact.
- The public route manifest covers the supported product inventory and every required
  route resolves from the locally built artifact and the deployed Pages site.
- The portal matches the React app’s documented tokens and Flat 2.0 rules in desktop
  and mobile light/dark views, with visible focus and no stock Docusaurus identity.
- One and only one GitHub Actions workflow publishes Pages, and it contains no routine
  development test jobs.
- The local frozen build, sanitizer, truth, link, route, generated-reference,
  responsive, keyboard, and accessibility gates pass after code/content completion.
- `.prometheus` history is represented only through reviewed, source-linked synthesis;
  sanitizer negative controls demonstrate rejection of raw logs, local paths, and
  secret-like material.
- Architecture/design and testing-methodology histories include reversals, failures,
  evidence limits, negative controls, and current authority.
- The live Pages root and representative deep links return the intended portal, and
  the repository homepage field and README point to that observed working URL.
- Every OpenSpec change validates with `openspec validate <change> --strict` and has
  row-form verification evidence scoped to documentation behavior only.

COMMANDS TO RUN
/opsx:new establish-documentation-publication-contract
/opsx:new repair-single-pages-portal
/opsx:new brand-uar-docusaurus-site
/opsx:new document-uar-theory-and-architecture
/opsx:new document-inference-skills-knowledge-and-agents
/opsx:new document-security-tenancy-governance-and-operations
/opsx:new document-apis-sdks-tools-and-deployment
/opsx:new reconcile-uar-readme-estate
/opsx:new publish-uar-architecture-decision-history
/opsx:new publish-uar-testing-methodology-history
/opsx:new certify-and-publish-uar-docs

SYCOPHANCY SELF-CHECK
- S-02: The plan is grounded in the measured assessment: it extends the existing
  Docusaurus site, addresses the observed publisher collision and npm/pnpm failure,
  and treats build health as unknown until the final local gate runs.
- S-07: The docs container, hosted search, analytics, and semantic rewriting of
  vendored/history content are explicit cuts rather than extra deliverables.
- S-03: Publication privacy, historical integrity, the no-partial-push constraint,
  and the trade-off of serializing shared site files are stated rather than hidden.

PLAN COMPLETE
