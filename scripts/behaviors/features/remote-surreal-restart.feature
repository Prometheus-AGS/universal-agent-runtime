Feature: UAR recovers when its remote SurrealDB server restarts
  A standalone UAR backed by a separate SurrealDB server must become ready
  again by itself after that server restarts, without restarting UAR (#441).

  Scenario: Readiness and persisted data return after a SurrealDB restart
    Given a standalone UAR backed by a remote SurrealDB server and an owned skill
    When the SurrealDB server restarts while UAR keeps running
    Then readiness returns to ready within 90 seconds without restarting UAR
    And the owned skill is still readable and a new skill can be saved
    And the UAR log records the transport loss and its recovery
