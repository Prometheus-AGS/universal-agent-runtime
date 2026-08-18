@ui @cross-screen @certification
Feature: Local-first browser resilience
  Browser-owned state must survive ordinary network and page lifecycle changes.

  Scenario: Offline state is visible and clears on reconnect
    Given the threads screen is ready
    When the browser goes offline and returns online
    Then the offline banner appears and then clears

  Scenario: PGlite retains a completed thread only in its browser context
    Given a completed deterministic chat is stored locally
    When I reload it and open a fresh browser context
    Then the original context retains the answer and the fresh context does not inherit it

  Scenario: The embedded SSE connection reconnects without duplicating runtime state
    Given the runtime cockpit has one known replayed run
    When the embedded sync stream disconnects and reconnects
    Then the restored cockpit still contains exactly one known run
