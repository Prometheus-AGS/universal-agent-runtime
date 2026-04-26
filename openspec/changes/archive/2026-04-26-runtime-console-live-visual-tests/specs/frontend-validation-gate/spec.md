## ADDED Requirements

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
