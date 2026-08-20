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
    Given a known knowledge base is visible through the registered embedded stream
    When the registered embedded sync stream reports an error and reconnects
    Then the knowledge screen contains exactly one recovered knowledge base
