@api @memory @restart
Feature: Memory retention and opt-out survive restart
  # E03. The host provisions one genuinely captured automatic memory whose
  # first capture is eight days old and a fresh automatic control memory
  # (scripts/acceptance/fixtures/provision-aged-memory.mjs). No clock override.
  Scenario: Shortening retention expires the aged memory, and a later opt-out survives restart
    Given the host-provisioned aged automatic memory and fresh control memory under a 30-day policy
    When I shorten the retention policy to 7 days
    Then the aged memory becomes unqueryable while the fresh control memory stays valid
    When I opt out of memory capture, tell the agent a new private fact, and the runtime restarts
    Then my opt-out survives the restart and the new fact was never captured
    And the aged memory stays gone after the restart while the control memory remains
