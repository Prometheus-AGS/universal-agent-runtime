# Decision log: fix-broken-session-configuration-ui

### 2026-08-23T20:03:09Z — Use the configured-provider source for Session Configuration

Decision: Opening Session Configuration will derive its model options from the configured-provider inventory exposed by `/api/providers`; it will not fetch or hydrate the complete `/api/models` catalog.

Rationale: The live catalog response is 2,611,291 bytes and the current path publishes 7,248 sequential graph updates, while the configured-provider response is 3,949 bytes and contains the set the control actually displays. The smaller source also preserves the local OpenAI proxy that the static catalog omits.

Provenance: observed installed-browser failure, live API measurements, and repository call-path trace.

### 2026-08-23T20:03:09Z — Pin the published entity package pair at 3.0.2

Decision: The product frontend will use exact npm versions `@prometheus-ags/prometheus-entity-management@3.0.2` and `@prometheus-ags/entity-graph-core@3.0.2` rather than resolving entity-management through `workspace:*`.

Rationale: The user requires the release version. The existing workspace link selects `3.0.0-rc.1`, and entity-management 3.0.2 declares core `^3.0.2` as a peer. Exact paired pins avoid prerelease selection and peer-instance drift.

Provenance: user requirement and npm registry metadata.

### 2026-08-23T20:03:09Z — Enforce a stricter render-state rule than React's default lint

Decision: Preserve the official React lint suite, add a UAR-local ESLint rule that rejects every state setter call from the owning component's synchronous render body, and reset session drafts through a keyed component boundary.

Rationale: React's official `set-state-in-render` rule deliberately permits guarded conditional updates. The observed component passes the current ESLint configuration, so enabling or upgrading that rule cannot prevent recurrence. A narrow local rule can reject this repository's prohibited pattern without adopting a broad lint suite.

Provenance: React official documentation, Context7 React 19 documentation, local ESLint execution, Vercel React Best Practices, and Composition Patterns.

### 2026-08-23T20:03:09Z — Use a bounded live browser contract as the regression control

Decision: Replace the existing body-visible-only scenario with a short local installed-service Playwright check covering response time, interactions, request volume, configured options, console output, and computed padding. Do not put the check in GitHub Actions and do not use a soak.

Rationale: The existing scenario can pass while Chrome is frozen because the page body remains visible. The regression must observe the operator-facing failure mode and complete in seconds.

Provenance: supplied screenshots, installed-browser reproduction, existing test inspection, Playwright official network documentation, and project testing policy.

### 2026-08-23T20:26:40Z — Supersede direct 3.0.2 adoption with an upstream correction release

Decision: Treat Entity Management 3.0.2 as the reproduced baseline and negative control, fix fetched-list ingestion upstream, release the corrected core/React pair, and make UAR consumption serially dependent on that release.

Rationale: Exact NPM tarballs, signed tag `v3.0.2`, and latest `origin/main` all perform one batch entity upsert followed by one `setEntityFetched` publication per row and a list-result publication. `createGraphTransaction` provides rollback but does not defer store writes. Switching UAR to the 3.0.2 hook unchanged would preserve an O(N) publication path.

Supersedes: The 2026-08-23T20:03:09Z decision to pin the published entity package pair at 3.0.2 as the final implementation state. The 3.0.2 evidence remains authoritative for the baseline.

Provenance: `.prometheus/research/fix-broken-session-configuration-ui-entity-state-architecture.research/`, upstream commits `f29a7016` and `e2521001`, and user authorization to fix the upstream project.

### 2026-08-23T20:26:40Z — Put unsaved session business state in a distinct graph entity

Decision: Use canonical `AgentSession` for committed server state and a distinct `AgentSessionDraft` entity for unsaved edits. Components subscribe to individual draft fields; save commits once and cancel discards only the draft.

Rationale: Component-local session configuration violates the project's explicit-business-state rule, while Entity Management's canonical patch map is a shared overlay visible to every subscriber and therefore cannot isolate uncommitted edits. A separate draft identity preserves inspectability and commit/cancel boundaries without pretending that necessary field rerenders can be eliminated.

Supersedes: The 2026-08-23T20:03:09Z keyed component-local draft recommendation. The stricter no-render-phase-update rule remains valid.

Provenance: exact Entity Management 3.0.2 source, React/Zustand official documentation, project architecture rules, and strict sycophancy screening.

### 2026-08-23T20:26:40Z — Repair the dead facade as part of the UI defect

Decision: Session Configuration is not complete until saved session state reloads and a real inference turn uses the saved session model. The frontend and backend will share typed field names. Unsupported context controls will be implemented end to end or removed.

Rationale: The current panel sends `model_override` and `context_strategy`; backend `AgentSessionConfig` accepts `model` and no context strategy; the frontend never loads persisted session config; the turn builder uses the agent model. Fixing responsiveness alone would leave a control that lies about task completion.

Provenance: current UAR source, archived BDD design disclosure, and the operator's requirement to use Entity Management properly.

### 2026-08-23T20:34:00Z — Run UAR and upstream repairs in parallel

Decision: Move UAR immediately from the workspace `3.0.0-rc.1` resolution to exact published 3.0.2 and implement the bounded configured-model/session path while the upstream atomic-ingestion repair proceeds independently. Adopt the corrected patch release if it is published before phase close.

Rationale: The panel needs twelve configured models, so the 7,248-row publication storm is removed by the configured-provider boundary even before the upstream repair ships. Making all UAR work wait for an NPM release would add calendar delay without protecting this panel. The upstream defect remains real and must still produce code/PR or an issue with reproduction.

Supersedes: The 2026-08-23T20:26:40Z decision insofar as it made UAR consumption serially dependent on the corrected release. It does not supersede the requirement to repair Entity Management upstream.

Provenance: second isolated Analyze review, live configured-provider count, and direct source verification.
