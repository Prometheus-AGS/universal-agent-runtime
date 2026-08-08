## Purpose

Define the frontend lint and typecheck validation gate required before runtime-console UI hardening work can be accepted.
## Requirements
### Requirement: Frontend lint gate
The frontend validation gate SHALL require `bun run lint` to exit successfully before runtime-console UI hardening work is accepted.

#### Scenario: Lint passes with no findings
- **WHEN** the operator runs `bun run lint` from the `frontend/` directory
- **THEN** the command MUST exit with status code 0
- **AND** the output MUST contain no ESLint errors or warnings.

#### Scenario: Lint failure blocks acceptance
- **WHEN** `bun run lint` reports any ESLint error or warning
- **THEN** the runtime-console validation-hardening phase MUST keep `frontend_lint_zero_warning` in a non-complete KBD state.

### Requirement: Typecheck preservation
The frontend validation gate SHALL preserve TypeScript project build health while resolving lint findings.

#### Scenario: Typecheck remains green
- **WHEN** lint fixes are applied
- **THEN** `bun run typecheck` from the `frontend/` directory MUST still exit with status code 0.

### Requirement: React effect safety
The frontend validation gate SHALL resolve React hook lint findings without disabling the hook rules globally.

#### Scenario: Effect state synchronization is refactored
- **WHEN** a component triggers `react-hooks/set-state-in-effect`
- **THEN** the implementation MUST use a safer pattern such as derived state, initialized state, asynchronous callback updates, or event-handler updates.

#### Scenario: Hook rule remains enabled
- **WHEN** ESLint configuration is evaluated
- **THEN** React hook rules MUST remain active for frontend TypeScript and TSX files.

### Requirement: Fast Refresh boundary hygiene
The frontend validation gate SHALL resolve Fast Refresh warnings without hiding future component boundary problems.

#### Scenario: Non-component exports cause warnings
- **WHEN** `react-refresh/only-export-components` reports a warning for a component module
- **THEN** the implementation MUST move reusable constants/helpers to a non-component module or add a narrow documented exception for an accepted UI-library pattern.

### Requirement: KBD progress update
The frontend validation gate SHALL update KBD workflow state when validation status changes.

#### Scenario: Lint gate is verified
- **WHEN** `bun run lint` and `bun run typecheck` both pass after this change
- **THEN** `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` MUST record `frontend_lint_zero_warning` as complete or verified.

### Requirement: Runtime console visual test gate
The frontend validation gate SHALL include targeted Playwright verification for the runtime console visual and navigation requirements introduced by `runtime-console-live-visual-tests`.

#### Scenario: Targeted runtime console Playwright suite passes
- **WHEN** the operator runs the targeted runtime console Playwright suite from the `frontend/` directory
- **THEN** the command MUST exit with status code 0
- **AND** the suite MUST cover desktop navigation, mobile navigation, command palette routing, and key runtime surfaces.

#### Scenario: Visual test failure blocks KBD completion
- **WHEN** the targeted runtime console Playwright suite fails
- **THEN** `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` MUST keep `runtime_console_visual_checks` in a non-complete state.

#### Scenario: Lint and typecheck remain required
- **WHEN** runtime console visual tests are added
- **THEN** `bun run lint` and `bun run typecheck` from the `frontend/` directory MUST still pass before this change is accepted.

### Requirement: Runtime replay validation gate
The frontend validation gate SHALL include targeted runtime event replay/entity-sync checks before runtime console validation-hardening work is accepted.

#### Scenario: Runtime replay validation passes
- **WHEN** the operator runs the targeted runtime event replay/entity-sync frontend validation command
- **THEN** the command MUST exit with status code 0
- **AND** the validation MUST cover runtime event normalization into the Prometheus entity graph.

#### Scenario: Runtime replay validation failure blocks acceptance
- **WHEN** the targeted runtime event replay/entity-sync validation command fails
- **THEN** the runtime-console validation-hardening phase MUST keep `runtime_event_replay_tests` in a non-complete KBD state.

#### Scenario: Existing frontend gates remain required
- **WHEN** runtime replay validation is added
- **THEN** `bun run lint`, `bun run typecheck`, and targeted runtime console Playwright validation MUST still pass before this change is accepted.

### Requirement: A2UI axe-core validation gate
The frontend validation gate SHALL run package-local axe-core and interaction tests when A2UI renderer source, styles, resources, stories, tests, or dependencies change.

#### Scenario: A2UI accessibility checks pass
- **WHEN** a pull request changes `frontend/packages/a2ui-uar/**`
- **THEN** CI runs the package typecheck, lint, tests, and axe-core fixtures successfully before acceptance

#### Scenario: Existing frontend gates remain required
- **WHEN** the A2UI accessibility gate is added
- **THEN** frontend workspace typecheck, lint, and build remain required for Change 21 completion

