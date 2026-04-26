## ADDED Requirements

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
