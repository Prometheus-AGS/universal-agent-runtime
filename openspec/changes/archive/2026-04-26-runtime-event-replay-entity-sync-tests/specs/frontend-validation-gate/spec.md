## ADDED Requirements

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
