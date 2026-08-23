# PLAN: fix-broken-session-configuration-ui

Project: Universal Agent Runtime, with one controlled Prometheus Entity Management upstream change
Date: 2026-08-23
OpenSpec available: YES
Changes to implement: 4

## Outcome

Ship the UAR fix on the exact published Entity Management/Core 3.0.2 release,
make Session Configuration responsive and effective for real inference, and
repair Entity Management's general N+2 fetched-list defect upstream without
making that unpublished patch a prerequisite for UAR.

## Why this is the shortest safe route

1. The release adoption is isolated and first, so no UI code is written against the stale `3.0.0-rc.1` workspace.
2. The UAR repair is one frontend/backend vertical slice; it does not wait for upstream publication and does not redesign unrelated model administration.
3. Guardrails and the single installed-service proof happen only after UAR code is complete, so implementation is not interrupted by a test campaign.
4. The upstream defect is fixed once at its true owner, with 3.0.2 retained as the negative control; UAR does not carry a fork or copied patch.
5. The phase excludes broad unit, regression, visual, load, and soak suites. Required Tier 0 checks remain cheap edit feedback; product behavior is checked once through short local HTTP/browser integration after code completion.

## Change list (ordered)

### 1. adopt-entity-management-3-0-2 — Resolve the reviewed release, not the workspace prerelease

- Scope: frontend dependency manifest; root/frontend pnpm lockfiles; dependency and boundary evidence
- Depends on: NONE
- Library: `cand-001`
- Recommended agent: Codex in `~/.claude/worktrees/uar-adopt-entity-management-3-0-2`
- Est. complexity: S
- Complexity score: Medium (five tracked tasks across two lockfile authorities; this routing score is intentionally higher than the S elapsed-time estimate)
- Model class: medium
- Customer value: HIGH
- Details: Pin both `@prometheus-ags/prometheus-entity-management` and its core singleton to exact registry 3.0.2. Prove both install roots resolve the reviewed integrity metadata and no application dependency is a local `link:` target. Do not edit the vendored Entity Management workspace or `versions.toml`.
- Permitted product files: `frontend/package.json`, `frontend/pnpm-lock.yaml`, `pnpm-lock.yaml`
- Completion evidence: registry/integrity resolution, `pnpm list`/`pnpm why`, local platform boundary, required Tier 0 checks, strict OpenSpec validation, row-form verification

### 2. repair-session-configuration-entity-flow — Replace the frozen dead facade with one effective entity flow

- Scope: entity platform/contracts/transports/domain hooks; chat session UI; configured model selector; typed session API; Rust effective model resolution
- Depends on: `adopt-entity-management-3-0-2`
- Libraries: `cand-004`, `cand-006`
- Capability gaps: `entity-transport-boundary`, `inspectable-session-draft`, `effective-session-inference`
- Recommended agent: Codex in a new worktree based on change 1's commit
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH
- Details: Register configured Provider/Model and AgentSession transports behind the platform facade, with a local graph-owned AgentSessionDraft and narrow field selectors. The sheet loads only `/api/uar/providers`, uses the typed `model` field, loads persisted state, and makes the saved session model participate in final inference routing. Remove unsupported decorative context controls unless their complete runtime contract is implemented. Apply the existing sheet spacing scale.
- Permitted product files: `frontend/src/platform/entities/**`, `frontend/src/entities/{bootstrap,schemas,types}.ts`, the Session Configuration component and its adjacent chat/model hooks, the configured model selector and adjacent model-domain code, obsolete chat session store/service files and their direct consumers, `src/uar/api/discovery.rs`, `src/server.rs`, and focused adjacent fixtures
- Completion evidence before the next change: required TypeScript and Rust Tier 0 checks only; a per-control inventory naming each retained control's typed API/persistence/runtime owner or removal diff; no product/unit suite yet; strict OpenSpec validation and row-form code-completion evidence

### 3. prevent-session-configuration-regressions — Encode the architecture and run one bounded proof

- Scope: active agent/TypeScript rules; frontend boundary checker and fixtures; one installed-service HTTP/browser scenario; phase evidence
- Depends on: `repair-session-configuration-entity-flow`
- Libraries: `cand-007`, `cand-008`
- Capability gap: `react-code-quality-instructions`
- Recommended agent: Codex in a new worktree based on change 2's commit
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: HIGH
- Details: Reconcile `AGENTS.md`, `CLAUDE.md`, and `.claude/rules/typescript.md` so server business entities use Entity Management while Zustand remains limited to transient UI/process state. Extend the existing local boundary gate with syntax-aware negative fixtures for the observed failure shapes. Once all UAR code is complete, build/install once and run a short functional scenario against `http://localhost:1906` covering responsiveness, request bounds, persistence/cancel isolation, model precedence, genuine inference, spacing, browser diagnostics, and matching server logs.
- Permitted product files: `AGENTS.md`, `CLAUDE.md`, `.claude/rules/typescript.md`, `scripts/check-frontend-boundaries.mjs`, `scripts/test-frontend-boundaries-negative.mjs`, `scripts/fixtures/frontend-boundaries/**`, `frontend/e2e/chat-session-config.spec.ts` or one narrowly named replacement, local Playwright configuration only if required to target the installed service, and `.prometheus`/OpenSpec/KBD evidence
- Completion evidence: observed failing negative fixtures, repaired source passing the local boundary gate, one short installed release HTTP/browser run, strict OpenSpec validation, row-form verification

### 4. fix-atomic-fetched-list-ingestion — Fix the N+2 owner upstream

- Scope: Prometheus Entity Management core graph/engine; React list/query/view paths; affected list adapters; changeset/version; upstream evidence and PR
- Depends on: NONE; runs independently of UAR Track A
- Build requirement: `cand-002`, capability gap `upstream-atomic-list-ingestion`
- Recommended agent: Codex in the existing clean worktree `/Users/gqadonis/.claude/worktrees/entity-management-fix-atomic-fetched-list-ingestion`
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH
- Details: Add one framework-neutral fetched-list ingestion action that merges rows, stamps lifecycle/sync state, and applies replace/append list metadata in one Zustand publication. Migrate every production list success path with the split-write pattern. After upstream code is complete, prove one publication at 1, 12, and 7,248 rows while unmodified 3.0.2 observes N+2, then follow fixed-group patch versioning, commit, push, and open the upstream PR. If safe code delivery is impossible, file the exact evidenced issue instead.
- Permitted product files: `packages/entity-graph-core/src/{graph,engine,index}.ts` and focused tests; `packages/entity-graph-react/src/hooks/**`, `packages/entity-graph-react/src/view/**`, affected list adapters and focused tests; package export/consumer fixtures only if the public surface changes; one repository-standard changeset; upstream OpenSpec/KBD/evidence
- Completion evidence: 3.0.2 negative control, 1/12/7,248-row positive controls, complete replace/append/lifecycle/sync/error assertions through the public fetch path, affected package/consumer checks after code completion, strict OpenSpec validation, row-form verification, isolated branch commit/push/PR

## Dependency and execution order

```text
Track A (UAR):  change 1  →  change 2  →  change 3
                                   └──── one post-code-completion functional proof

Track B (upstream): change 4 ────────────────────────────────┐
                                                             └─ required deliverable,
                                                                never a UAR blocker
```

- Round 0: stage only this phase's reviewed planning artifacts as an isolated planning baseline; preserve all unrelated dirty paths. If that baseline cannot be isolated, stop rather than sweeping unrelated files into a commit.
- Round 1, parallel-capable: changes 1 and 4. With one primary executor, start change 1 first because it immediately unblocks the UAR critical path, then continue change 4 in its already-created upstream worktree without modifying the dirty primary checkout.
- Round 2: change 2 begins from change 1's commit. Change 4 may continue independently.
- Round 3: change 3 begins from change 2's commit and owns the only UAR product-behavior verification run.
- Round 4: reconcile the three UAR commits forward to `main`; confirm the upstream PR/issue exists; reflect only when every per-requirement result is recorded.

UAR remains on exact 3.0.2 throughout this phase. A later published upstream
patch is adopted only through a separate reviewed dependency change, never by
silently changing this phase's pin.

## Worktree and commit discipline

- Keep one worktree per change under `~/.claude/worktrees/`; never create one inside either repository.
- Track A rebases forward from the preceding commit and never merges sideways.
- Commit each UAR change independently; do not push UAR or open a UAR PR without new operator authority.
- Track B uses `codex/fix-atomic-fetched-list-ingestion` based on updated upstream `origin/main`; never edit/reset the dirty upstream primary checkout.
- Track B's user-authorized deliverable includes its versioned commit, push, and PR after verification.
- Preserve `.prometheus` history, `.claude/settings.local.json`, `versions.toml`, and unrelated changes.

## Implementation and verification discipline

### While code is being written

- Run only the required Tier 0 checks for the file stack: TypeScript typecheck/lint and scoped Rust `cargo check` as defined in `.claude/rules/`.
- Do not run unit, broad integration, end-to-end, visual, performance, load, stress, soak, or release tests.
- Complete one owned edit and its cheap check before the next subsystem.
- GitHub Actions remain deployment-only and are not changed by this phase.

### After UAR code completion

Run one bounded local functional sequence, not a broad suite:

1. Build/install the production UI and service once at the existing local deployment.
2. Exercise direct HTTP and a real browser at `http://localhost:1906`.
3. Observe explicit-turn, saved-session, and agent-default model precedence.
4. Open Session Configuration within two seconds; prove no `/api/models` request and no more than configured provider count + configured model count + six graph publications.
5. Select a configured model, save, reopen, and complete genuine inference with that effective model.
6. Change a draft and cancel; prove committed state is unchanged.
7. Inventory every visible control and record its typed API, persistence, and runtime owner; any control without all three must be removed and evidenced by its diff.
8. Capture computed insets at the established 320, 768, 1024, and 1440 pixel certification widths, plus console/network evidence and the matching `.prometheus` server-log interval.
9. Run strict OpenSpec validation for the three UAR changes.

Any unmet result remains a per-requirement failure. There is no aggregate
percentage or runtime-level readiness verdict.

### After upstream code completion

- Run the public fetched-list/store subscription integration fixture at 1, 12, and 7,248 rows.
- Preserve unmodified 3.0.2's observed N+2 output as the negative control.
- Run only affected package/adapter/packed-consumer checks required by the changed export surface.
- Version through the repository's fixed-group changeset path, then strict-validate, commit, push, and open the PR.

## Stop conditions

The eleven stop conditions in `spec-index.md` are binding. The most likely are:

- registry 3.0.2 does not resolve as one core singleton from both UAR roots;
- final model precedence cannot be changed without breaking explicit request behavior;
- a retained UI control has no runtime owner and an active requirement forbids removing it;
- the upstream action changes data/list/error semantics beyond the specified shared batch timestamp;
- any required file falls outside the permitted surfaces;
- the dirty upstream primary checkout would need to be touched.

## Adversarial review carry-forward

Spec review round 2 passed with zero critical findings. Four warnings remain:

1. The canonical scenario name `Adapter boundaries are checked in CI` is legacy and misleading. OpenSpec strict validation refuses a MODIFIED block that renames it; its normative body explicitly requires local-only execution and prohibits GitHub Actions.
2. The circular functional-gate wording was corrected: the browser run follows changes 1/2 and change 3's code/fixture completion.
3. Upstream batch timestamp semantics now explicitly replace incidental per-row clock sampling while preserving freshness ordering between fetches.
4. Analyze evidence and inspected source paths are indexed in `spec-index.md`; execution must reverify them and stop on drift.

The non-blocking measurability suggestion was also resolved with a concrete
configured provider + configured model + six publication ceiling. Plan-review
warnings were resolved by adding that bound to the final proof, adding the
per-control runtime-owner inventory, and using the repository's established
320/768/1024/1440 responsive certification matrix.

## Commands represented by this plan

The Spec stage already emitted and strict-validated these structures; do not run
`/opsx:new` again:

```text
/opsx:apply adopt-entity-management-3-0-2
/opsx:apply repair-session-configuration-entity-flow
/opsx:apply prevent-session-configuration-regressions

# In /Users/gqadonis/.claude/worktrees/entity-management-fix-atomic-fetched-list-ingestion
/opsx:apply fix-atomic-fetched-list-ingestion
```

## Uncomfortable trade-off

The quickest UAR fix intentionally ships on a package version known to retain a
general large-list defect. That is acceptable only because this sheet is
contractually cut off from the 7,248-row catalog and bounded to the configured
set, while the upstream defect remains a required, separately evidenced phase
deliverable. Dropping either side would turn the plan into a patch or a delay.

PLAN COMPLETE
