Feature: Governed media effects through the UAR collaboration API
  Background:
    Given a durable UAR media fixture with externally supplied identities

  Scenario: Claims and cancellation serialize and survive UAR restarts
    Given a catalog snapshot from before the media fields were introduced
    When prepared media intent survives a UAR restart
    And concurrent duplicates compete for dispatch and a mock result is lost
    And UAR restarts before the result is recorded
    Then reconciliation settles the original attempt without another peer invocation
    And cancellation races dispatch without implying an external undo
    And raw meeting content is absent from persisted media state and diagnostics

  Scenario: Exact authority governs meetings and simulated call takeover
    When a coordinator and delegated worker perform approved meeting actions
    Then the mock peer records each meeting effect once
    When the host performs approved simulated call start, takeover and end
    Then the mock peer records each call effect once

  Scenario: Scope, approval and worker boundaries reject unauthorized effects
    When the host and untrusted callers submit unauthorized media operations
    Then the durable catalog contains only the authorized preparations and no dispatch attempts
