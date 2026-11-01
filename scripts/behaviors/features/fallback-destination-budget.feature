@api @context @budget @failover
Feature: A failover destination is compiled against its own context window (#388)

  When the primary model is unavailable and a configured fallback has a smaller
  context window than the request was prepared for, the fallback request is
  either displaced to fit that window by the agent's declared strategy, or it is
  refused with a typed budget error before any request reaches the fallback.
  The test never sends credentials: the caller is the loopback owner.

  Scenario: A smaller fallback window receives a displaced request that fits
    Given a dedicated UAR with an unavailable primary and a fallback with a 4096-token window
    And a disposable agent using the keep-first-and-latest strategy
    When I continue a seeded 60-message conversation that exceeds the fallback window
    Then the run is answered by the fallback
    And the fallback request was displaced to fit its own window

  Scenario: A fallback that cannot hold the request is refused before dispatch
    Given a dedicated UAR with an unavailable primary and a fallback with a 4096-token window
    And a disposable agent that never displaces history
    When I continue a seeded 60-message conversation that exceeds the fallback window
    Then the run ends with a typed context overflow refusal
    And the fallback provider received no request
