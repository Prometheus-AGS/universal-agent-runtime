## ADDED Requirements

### Requirement: Strict baseline gating

The eval runner SHALL support a strict mode in which a missing baseline is a
failure rather than a pass, so a regression gate cannot silently pass before a
baseline has been established. The scheduled CI tier SHALL run in this strict
mode. Strict mode SHALL be opt-in; without it, a run with no baseline remains
clean (a run may still establish expectations).

#### Scenario: Strict run with no baseline fails
- **WHEN** `eval run <suite>` is invoked with the strict-baseline option and no baseline exists for the suite
- **THEN** the process prints a clear message and exits non-zero, without reporting a (vacuous) clean comparison

#### Scenario: Non-strict run with no baseline is clean
- **WHEN** `eval run <suite>` is invoked without the strict-baseline option and no baseline exists
- **THEN** the run completes and exits zero (unchanged prior behavior)

#### Scenario: Scheduled tier is strict
- **WHEN** the scheduled eval workflow runs its gating step
- **THEN** it uses strict-baseline mode, so a missing committed baseline fails the job until a baseline is seeded
