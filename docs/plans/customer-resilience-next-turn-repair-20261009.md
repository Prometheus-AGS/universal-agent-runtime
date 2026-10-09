# Persisted resilience policy at team admission

## Observed failure and source gap

The published 2.2.25 reviewed-skills attempt `reviewed-skills-imports-52f7acb2-46f9-4f17-ad33-3da7e56a9ebb` reports `TEAM_PROVIDER_REQUEST_REJECTED`, stage `provider-opening`, category `provider_timeout`. The coding retry `coding-team-retry-f27d7acd-446d-4ee4-8598-13991bb7879c` also records provider rejection, but does not preserve the typed timeout category. Both failed attempts remain historical evidence. Ordinary UAR inference passed.

At source 7183a0af6db1b1a6385bcd5b36b591aad504b8d7, settings administration labels resilience changes `next_turn`, but RunManager clones its startup resilience policy into each orchestrator. Only the ordinary HTTP wrapper loads persisted global and per-agent policies. Consequently a persisted change cannot affect team provider-opening deadlines.

## Repair

Extract the existing typed settings resolver to `src/uar/settings/persisted_resilience.rs` and reuse it in the HTTP wrapper and RunManager. Share the initialized server SettingsManager with RunManager using its existing builder. Capture the resolved global/per-agent policy once after the admitted artifact is selected; pass that immutable snapshot to the orchestrator, including team attempts, hosted descendants and continuation turns. Settings changes affect the next admitted turn, not an already-running provider request.

Keep all existing policy fields, override semantics, validation/fallback behavior, startup defaults and dependency versions. No new REST interface, persistence table, timeout heuristic or automatic timeout widening. No credentials or protected provider diagnostic bodies enter this document.

## Deadline attribution and remaining evidence

UAR defaults remain 15 seconds for driver stream establishment and 30 seconds for the first semantic event. Both errors are categorized as provider-opening timeout; the existing safe receipt does not identify which expired. Liter's configured HTTP request timeout is a separate deadline. A provider rejection alone does not prove entitlement failure or support changing timeout values.

Compilation and actual packaged operation of the corrected source are pending the completed delivery boundary. No Cargo command, suite, application launch or verification build was run for this repair. The affected operation should first demonstrate that an explicitly persisted resilience setting reaches the team orchestrator, then operate the previously failed coding/reviewed-skills path with the configured model. Passing that operation does not certify unrelated portfolio criteria.
