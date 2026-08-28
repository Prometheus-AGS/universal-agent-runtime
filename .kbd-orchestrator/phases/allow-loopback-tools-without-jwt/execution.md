# EXECUTION: allow-loopback-tools-without-jwt

Project: Universal Agent Runtime
Date: 2026-08-27
Selected backend: openspec
Dispatched to: Codex self-execution
Backend rationale: The phase is one security-sensitive vertical slice with approved OpenSpec deltas, a 42-task working checklist, runtime/frontend/deployment boundaries, and explicit verification tiers. OpenSpec provides the required task-level traceability while the canonical KBD runtime records cohesive work-package progress.
Backend entrypoint: `/kbd-apply allow-loopback-tools-without-jwt`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/allow-loopback-tools-without-jwt/plan.md`

## EXECUTION SCOPE

- `allow-loopback-tools-without-jwt`: make governance optional only for a boot-verified loopback-only, JWT-disabled process; persist and publish the live master state; expose truthful operator controls; certify and deploy the release.

## DISPATCH CONTRACT

- Codex executes one OpenSpec checkbox and its cheapest applicable check at a time.
- `openspec/changes/allow-loopback-tools-without-jwt/tasks.md` is the task-level working surface.
- `.kbd-orchestrator/phases/allow-loopback-tools-without-jwt/progress.json` is the canonical cross-tool work-package ledger and is mutated only through `prometheus kbd` commands.
- On each work-package boundary, transition the corresponding KBD task with an evidence summary and refresh exact next work.
- Preserve every unrelated dirty worktree file and completed historical KBD phase.

## APPROVAL GATES

- Tier 3 milestone/release execution: already authorized by the operator in this task.
- Release binary replacement and LaunchAgent restart: explicitly requested by the operator; execute only after required local gates pass.
- Commit, push, and PR creation: explicitly requested; commit only scoped files and create/update PRs only for affected repositories not already on main.
- OpenSpec archive: only after all implementation and tracked work, including reflection and publication, is complete.

## FALLBACK CONDITIONS

- Stop and revise the plan if the runtime cannot enumerate and seal every tool-capable ingress before admission, if persistence cannot provide the designed durable ordering, or if existing API batch semantics cannot represent truthful per-key outcomes without an incompatible public contract.
- Fail closed to effective governance On with mutation unavailable for unresolved persistence; do not substitute an in-memory Off default.
- If a required verification command exposes an unrelated baseline failure, record its exact output and distinguish it from a regression before continuing.

## VERIFICATION REQUIREMENTS

- Rust Tier 0: `cargo check --locked --no-default-features --features server-full` after each cohesive Rust edit.
- Rust Tier 1: only focused new unit/integration tests for the completed work package.
- Frontend Tier 0: `pnpm typecheck` and `pnpm lint` after cohesive frontend edits.
- Frontend Tier 1: focused settings API/store/hook/panel tests.
- Tier 2: exact Rust format/full locked server-full tests and exact frontend build/full test/policy checks from OpenSpec task group 8.
- Tier 3: focused Governance Playwright matrix, supported-profile/release certification, and `cargo build --release` under the existing single-writer target discipline.
- Deployment: exact binary identity, LaunchAgent process restart, authoritative status, warning cardinality, local default/toggle/tool success, and JWT/non-local fail-closed proof.

## PROGRESS LEDGER

- DONE — work package 1, workflow and normative alignment: KBD Assess/Plan revisions recorded and strict OpenSpec delta validation passed.
- IN_PROGRESS — work package 2, fail-closed runtime authority and boot boundary.
- PENDING — work packages 3–9.

## OUTPUTS

- OpenSpec proposal, design, two modified capability deltas, and 42-task checklist.
- KBD assessment, plan, execution, canonical progress, and waypoint projections.
- Production runtime, settings/API, frontend, tests, release artifact, deployment receipts, reflection, and archive produced during later tasks.

## BLOCKERS

- None. The local KBD HTTP control plane is unavailable, but every canonical command has committed through the supported local-runtime fallback; remote control-plane status remains unknown and is not an implementation blocker.

## REFLECTION HANDOFF

- Compare delivered behavior and files against all 42 OpenSpec tasks and the nine KBD work packages.
- Lead with deltas from the plan, including any boot-order compromise, API contract change, baseline failure, unverified downgrade target, or deployment limitation.
- Carry exact Tier 0–3 and live deployment receipts, UI review receipts, unrequested additions, trust-boundary guards, and remaining risk into reflection and `.prometheus/` append-only memory.

## EXECUTION READY

## UI/UX IMPLEMENTATION RULES — 2026-08-27

The Governance surface is an incumbent React settings panel in **Operate** mode. Impeccable context, audit, and critique guidance; Anthropic `frontend-design`; UI/UX Pro Max targeted UX/React searches; Vercel React Best Practices; and Prometheus Entity Graph Realtime were consulted before React edits. The separately installed Vercel Composition Patterns and `ux-designer` skills were not present at any installed skill root, so the repository's explicit composition and component→hook→store→service rules remain the controlling fallback.

Apply these task-specific rules:

- Preserve the existing visual system and Save/Refresh commit boundary. Make the hierarchy product-specific through truthful enforcement state, not a new palette, font, modal, or decoration.
- Lead with a dedicated master row labeled **Enforce tool governance** using `minmax(0, 1fr)` and wrapping copy/actions. The authoritative badge, persistent Off warning, neutral draft note, assertive error, and one atomic polite announcement are separate regions.
- Never derive effective enforcement from a setting value or draft. Ingest one coherent server-confirmed Governance status record into the normalized entity graph and expose it through a focused domain hook with boot-instance, revision, and request-order acceptance.
- Keep saved settings, per-key drafts, saving state, mutation results, and effective runtime status separate. A partial response reconciles every key independently; unchanged successful drafts clear, failed/skipped/dependency-failed and post-submit edits remain.
- Subscribe at the smallest practical rendered boundary. Components import only feature hooks and shared UI primitives; services own fetch/deadlines, stores own graph ingestion and sequencing, and components never import transports or the graph store.
- Use semantic labels and descriptions for every select, switch, and action group. Required uses a focusable guarded `aria-disabled` master so keyboard and assistive-technology users can reach the reasons; dependent policy controls live in a real `fieldset`/`legend` and are natively disabled when the draft master is Off or while submitted.
- Keep keyboard order aligned with visual order, preserve visible focus, announce one meaningful contextual status without moving focus, place recovery actions with errors, and avoid visual-only state.
- At 320 CSS px and 200% zoom, allow text and actions to wrap, set shrinkable children to `min-width: 0`, avoid blind `overflow-hidden`, and prevent page/card/fieldset horizontal overflow. Keep semantic theme tokens for light/dark contrast.
- Use event handlers for interaction work, primitive effect dependencies, cleanup for focus/reconnect/change subscriptions, functional state updates, and no inline component definitions.

Isolated pre-edit critique receipts:

- `governance_ui_critic_a`: P0 missing authoritative master/effective state; P1 generic draft/effective collapse, policy-first hierarchy, and non-focusable Required control; P2 understated navigation copy. Recommended a dedicated single-column master-detail form.
- `governance_ui_critic_b`: P0 absent status/master contract; P1 partial-save rollback and post-submit draft loss, unnamed controls, unsupported Required keyboard behavior, non-authoritative async feedback, and missing focused tests; P2 broad subscriptions/row-by-row ingestion and narrow-layout risk. Detector was intentionally reserved for the completed UI.

## IMPLEMENTATION EVIDENCE — 2026-08-27

### Runtime, settings, and tool boundary

- Added separate governance mutation, gate, and status authorities; exact configured-literal and sealed bound-ingress eligibility; inactive admission tokens; fail-closed initialization; coherent boot/revision snapshots; and a process-scoped inactive warning.
- Added the durable `governance.enabled` setting, one Governance namespace mutex, ordered batch results, posture normalization, reset semantics, authoritative status and mutation responses, and realtime notification after commitment.
- Added `GovernanceBypassed` and read the coherent status before policy, Cedar, risk, and approval gates. Registration, argument, transport, provider, and execution failures remain downstream and unchanged.
- Final focused Rust commands:
  - `cargo test --locked --no-default-features --features server-full --lib governance` — exit 0; 21 passed, 0 failed, 645 filtered out. The compiler emitted four pre-existing warnings.
  - `cargo test --locked --no-default-features --features server-full --test settings_persistence governance` — exit 0; 6 passed, 0 failed, 38 filtered out. The compiler emitted the same pre-existing warnings.
  - The earlier broad filtered command reached the same 21/21 library result but exited 2 because the standalone `tests/bdd.rs` harness rejects positional test filters. It is not counted as a passing receipt.
- Tasks 2.6, 3.5, and 4.5 remain open because their contracts require zero warnings. No warning was hidden or repaired outside scope.

### Frontend state, panel, and UI quality

- Added strict status/mutation contracts, normalized status ingestion, boot-instance/revision/request ordering, live refetch triggers, bounded confirmation deadlines, per-key draft reconciliation, and terminal confirmed/partial/rejected/Unknown save states.
- Rebuilt the existing Governance panel as the approved master-detail surface. The authoritative badge, persistent Off warning, draft note, polite transition announcement, and assertive error stay distinct. Required is focusable and guarded; dependent policy uses a semantic disabled fieldset.
- Corrected the observed narrow-settings defect by bounding the mobile navigation height, allowing the master row to stack below 360 CSS px, making the Off warning boundary explicit, and checking overflow/focus against the actual enclosing regions.
- Final focused component/API/store/hook tests passed: 2 focused settings/Governance files, 20 tests. `pnpm typecheck`, `pnpm lint`, and `pnpm build` exited 0; the production build retained four existing PGlite direct-eval warnings.
- The authorized production Playwright Governance matrix passed 5/5. It covered 320 effective CSS px, the 200% zoom equivalent, light/dark themes, computed contrast, page/card/fieldset containment, focus visibility, keyboard toggling, Required reasons, Unknown recovery, delayed post-restart confirmation, and authoritative On/Off transitions.
- Two isolated Impeccable critics passed after remediation. The manual detector was run exactly once and returned no finding. A fresh-context adversarial review found mobile-shell, contrast-boundary, zoom-oracle, response-authority, containment, Required-reason, and focus-proof gaps; each was corrected. The final fresh-context adversarial review returned PASS with no critical or warning blocker.

### Tier 2 and Tier 3 truth

- Rust Tier 2 is not green: `cargo fmt --all -- --check` passed, but the exact locked server-full suite ended with 663 passed, 2 unrelated routing-evaluation failures, and 1 ignored, plus existing warnings. Task 8.1 remains open.
- Frontend Tier 2 is not green: production build passed, but the full suite retained 77 passing files/3 failing files and 391 passing tests/12 unrelated provider-mock and A2UI Storybook failures. Typecheck, lint, GitHub Actions policy validation, and strict OpenSpec validation passed. Task 8.2 remains open.
- Tier 3 browser proof passed. `pnpm release-local-contracts:validate` and `pnpm github-actions-policy:validate` passed. `pnpm support-matrix:validate` failed because the existing product-support matrix omits the Cargo `embedded-mobile` feature. Task 8.4 remains open.
- `cargo build --release --locked --no-default-features --features server-full` completed in 18m45s. The version is 1.0.0, size 211,763,392 bytes, SHA-256 `a7aefee1d23be3b0f65a08d07fcbfb9f8a8d50746035f08cc724543acb8ff42f`. It retained the existing unused-constant, missing-Debug, and future-incompatibility warnings.

### Installed release proof

- The supported macOS installer atomically installed the release to `/Users/gqadonis/.uar/bin/universal-agent-runtime`, replaced static assets, and restarted `com.prometheus.universal-agent-runtime`. The installed digest exactly matches the release digest above. The previous executable is recoverable at `/Users/gqadonis/.prometheus/backups/uar/universal-agent-runtime.pre-governance.20260827T143316Z` with SHA-256 `e7da66bc82811edbacee0a3a6177bb87f7c8b80fbe91e224f3c68e6e54c96e4a`.
- The service restarted at PID 77605 with HTTP on `127.0.0.1:1906` and `[::1]:1906`, plus gRPC on `127.0.0.1:50051`. Health passed; readiness reported Postgres and MCP ready, SurrealDB not configured, and six MCP tools.
- The first authoritative status was Off, mutable, JWT-disabled, and configured for `127.0.0.1`. Live On and Off mutations advanced revisions 10→11→12 with updated per-key results and matching status tokens. The inactive warning appeared exactly once in the installed process log.
- An installed anonymous MiniMax run selected and executed native `web_fetch` against `https://example.com`. The run completed with `decision_source="governance_disabled"`, HTTP 200 tool output, and zero approval/denial events (`c12f753d-9840-473e-8b16-2dcecc376603`).
- A fresh isolated loopback process defaulted Off, warned once, toggled On→Off live, and shut down cleanly. A fresh `0.0.0.0` process reported Required with `configured_host_not_allowed` and `bound_ingress_not_loopback`, rejected the Off mutation, and shut down cleanly. JWT-required fail-closed behavior is covered by focused deterministic tests, not a live authenticated process. Task 8.5 therefore remains open rather than overstating the live matrix.

### Rollback and downgrade evidence

- A temporary fail-closed source variant was release-built after the forward deployment, copied to ignored staging as `target/release-rollback/universal-agent-runtime`, and hashed as `e365af47cbd34f96aca88410892aa0fb3e55a1d9743c77b8cacc173e0c5f1be5`. The temporary edit was immediately reversed; `cargo fmt --all -- --check` passed, and `target/release/universal-agent-runtime` was restored byte-for-byte to the forward SHA-256.
- On a fresh isolated SurrealKV database, the rollback artifact reported effective On with `mutation_available=false` and reason `persistence_unavailable`; an Off request returned a per-key `validation_rejected` result and the status remained On.
- The pre-governance 1.0.0 binary then booted and passed health against the same database, returned 404 only for the intentionally absent status endpoint, and listed its known Governance rows without failing on the unknown `governance.enabled` row. Restarting the forward artifact proved that row was still present and readable as `true`.
- Reversible row procedure for a stopped service: export the complete provider row before mutation (`SELECT * FROM settings WHERE key = 'governance.enabled'` for SurrealDB, or the equivalent parameterized PostgreSQL query); retain the JSON/SQL export and checksum; delete only `settings:governance_enabled` in SurrealDB or `DELETE FROM settings WHERE key = 'governance.enabled'` in PostgreSQL; deploy the prior binary; and, on cancellation, restore the exported row exactly before restarting the forward binary. Never use a broad settings-table delete. The ordinary DELETE settings API is not this procedure because Governance reset intentionally writes the posture-derived default.
- Task 8.3 remains open: this proves the fail-closed behavior and the one locally available downgrade target, but the rollback artifact was built after forward deployment and was not committed as a release deliverable. That is a plan-order delta, not a passing pre-deployment rollback receipt.

## SOURCE COMPLETION FREEZE — 2026-08-27

- The operator corrected the execution sequence to code-to-end, then test. The active exact Rust Tier 2 process was terminated before completion; its partial output is not a verification receipt. No further test, check, build, lint, validation, browser, certification, or live-runtime command may run until the complete forward and rollback source/artifact set is frozen and the operator explicitly reauthorizes verification.
- Forward release-candidate source is `44fc519c7d65e0f125b812caf992121cf51c38ad`. Rollback release-candidate source is `ce712ee4a969d15d9c73533ae5be4266abdaea1f` on `codex/governance-rollback`. Each candidate includes the fail-closed rollback contract; the rollback production tree differs from forward only in `src/server.rs`, where startup forces governance On and publishes mutation unavailable.
- A source-only audit found no `TODO`, `FIXME`, `todo!`, or `unimplemented!` marker in the change-owned governance production paths. The remaining `PENDING` entries are evidence slots in `rollback.md` plus canonical work packages 8–9; they are not unfinished product branches.
- No OpenSpec verification checkbox was advanced from the aborted run. Binary digests, rollback compatibility, final Tier 0–3 receipts, installation, reflection, archive, publication, and PR evidence remain pending.

## FINAL CERTIFICATION — 2026-08-28

- The operator authorized the end-of-work Tier 3 gate. The consolidated forward
  candidate is `8b5ac5ea563e2c3eef03f55df2347b93e18942b8`; the fail-closed rollback
  candidate is `4582ed3aec793f5bc4d45097604fb761889295ea`.
- Exact Rust Tier 0 and Tier 2 passed with zero compiler warnings. The locked
  `server-full` suite passed 665 library tests with one ignored, 9/9 BDD
  scenarios and 49/49 steps, 93 integration tests with one ignored, 44 settings
  tests, one skill-pack test, nine UAR integration tests, and 17 doctests with
  17 ignored.
- Frontend Tier 2 passed: 80 files and 406 tests, typecheck, lint, production
  build, and GitHub Actions policy validation. Strict OpenSpec validation passed.
- Tier 3 passed: the focused Governance browser matrix was 5/5, the support
  matrix reported 23 features and nine provider tiers, release-local contracts
  passed with six MCP negative controls, and exact default-profile forward and
  rollback `cargo build --release` builds passed.
- The installer-profile `server-full` SHA-256 values are forward
  `0030737d255770c03d75e8f80faa51ebb436d25f02e646c33a96e8423ba24bff`
  and rollback
  `f725a77fc1fd24763bb55d2137fcaa90f8e5c4baaf4831a3515ac7500d525189`.
  The installed binary matches the forward digest exactly.
- The LaunchAgent is running the installed binary. Health and readiness pass.
  The installed authoritative status is Off at revision 12, JWT-disabled,
  loopback-only, and mutable. Live On→Off transitions passed, and the inactive
  warning occurred exactly once for the boot.
- Installed configured-tool execution passed with `native__memory_list` while
  governance was Off. The installed registry has no search MCP; search-specific
  bypass is therefore certified by the deterministic `web_search` integration
  regression, not misreported as a live third-party search call.
- Live non-local and JWT-required processes remained Required/On with exact
  reasons and no inactive warning. The rollback candidate remained On and
  mutation-unavailable.
- The shared-database downgrade exercise found that rollback normalizes an Off
  preference to On. Returning forward retains On until the exported preference
  is restored. This observed behavior supersedes the earlier row-preservation
  assumption and is now explicit in `rollback.md` and `certification.md`.

## CORRECTED FINAL CERTIFICATION — 2026-08-28

- The first isolated final-artifact review returned FAIL. It identified a stale
  rollback ownership claim, an implicit settings-notification race, missing
  direct-HTTP Cedar bypass coverage, stale progress, and no durable installed
  tool receipt. Those findings supersede the earlier candidate receipts above.
- The corrected settings path now commits durable storage, cache, and coherent
  runtime state before publishing one explicit realtime event. Implicit database
  Governance notifications are suppressed; delivery failure is logged after
  commitment and does not roll back the accepted setting.
- The direct HTTP tool middleware now consumes the same coherent Governance gate
  as `RunManager`. Focused tests prove Off bypass with `X-Agent-Id` and prove On
  still returns HTTP 403 through Cedar.
- Corrected focused gates passed with zero compiler warnings: 25 Governance
  library tests and eight Governance persistence/publication tests. The complete
  locked `server-full` suite passed 669 library tests with one ignored, 9/9 BDD
  scenarios and 49/49 steps, 93 integration tests with one ignored, 47 settings
  tests, one skill-pack test, nine UAR integration tests, and 17 doctests with
  17 ignored.
- Frontend production build, 80 files/406 tests, typecheck, lint, GitHub Actions
  policy validation, strict OpenSpec, the 5/5 Governance Playwright matrix,
  support-matrix validation, and release-local contracts all passed.
- Final build identities are forward source `5753cb19eacb2c562320a5bd941c330c2c9cb789`
  with default SHA-256 `9ad253b4e2b99b4cbdfeb1bd3dcf9b502a5217abc8e692c27055c211e871e0c1`
  and installer SHA-256 `901317098d77bdd8c9858e4751728e221f474ed0f3fe93f5600ffb7ac4dcbbe9`;
  rollback source `4e6fc087224975b6c5993c5386d3c3c7b20e24cd` with default SHA-256
  `56bad97f0d1abd674e24d171e9793d4725f5ebf30d2ba0165fb52c27f23fb84b`
  and installer SHA-256 `3959dc3d1fed7b4d9a31d59a4d8839816e7d992e235553454417890a29434b96`.
- The installed LaunchAgent runs the corrected forward digest at PID 15007.
  Health and readiness returned HTTP 200. Boot
  `18193a24-42a1-4276-ae72-26d29d2db5db` advanced Off 10 → On 11 → Off 12,
  warned exactly once, and executed `native__memory_list` successfully with an
  agent identity while Off. Current-source non-loopback and JWT-required live
  processes remained Required/On; the current rollback remained On and rejected
  mutation. Machine-readable receipts are under the change's `evidence/` folder.

## FINAL POST-CRITIC-2 CERTIFICATION — 2026-08-28

- A second fresh isolated critic found that the initial HTTP Cedar correction
  bypassed every action behind the application-wide middleware while Off, and
  that the recovery directory still held only superseded binaries. Both were
  release blockers; the earlier corrected-final identities are superseded.
- The final middleware bypass is restricted to POST `/api/tools/*/execute`.
  Off direct-tool execution passes; the same tool is denied while On; actor
  creation with an agent identity remains Cedar-governed and denied while Off.
  The three focused regressions passed, and the full required gate restarted.
- Final exact Rust results: zero-warning check, 26/26 focused Governance tests,
  8/8 focused persistence/publication tests, 670 library tests passed with one
  ignored, 9/9 BDD scenarios and 49/49 steps, 93 integration tests passed with
  one ignored, 47 settings tests, one skill-pack test, nine UAR integration
  tests, and 17 doctests passed with 17 ignored.
- Final frontend/release results: production build, 80 files/406 tests,
  typecheck, lint, policy validator, strict OpenSpec, Playwright 5/5, 23-feature
  and nine-provider support matrix, ten disabled dependency boundaries, and
  release-local contracts with six MCP negative controls all passed.
- Final forward source is `171cbf8531534c7c56fd72aea2a9c815172e85dd`:
  default SHA-256 `e5e1690de5e92a9c3b49f3ab15820cd073c1b78e86b6f082f2f8170ca3881f14`,
  installer SHA-256 `b6fe01c4f3e68e02ce5967da48d70d980880e01261a7c9d64bf8619e89450de2`.
  Final rollback source is `0f97859f56bf9f097ba8ecc78b24daff6612145a`:
  default SHA-256 `d3975af6e8fb068404a990e0a598f7241a1d5a5ddea0ff3affe7c5c959f8a0ca`,
  installer SHA-256 `4ff9e1157a139a30c7cc988e56afbe82e07907bf746293ae38ba32e05c5cbdcd`.
- Commit-qualified forward and rollback binaries are retained in the recovery
  directory. The installed LaunchAgent runs the final forward digest at PID
  45385. Boot `586bdaff-c660-43f2-a9d4-9c2e86119593` is healthy, ready, Off at
  revision 12 after On→Off, and warned once. The final rollback boot
  `d953564f-c0b7-4950-8e73-ce17cc92e6ba` remained On, mutation-unavailable,
  and rejected Off without changing revision 10.
