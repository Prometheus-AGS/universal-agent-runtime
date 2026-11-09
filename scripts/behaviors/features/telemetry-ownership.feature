@api @telemetry @owned-journals
Feature: Mounted telemetry retains attribution without private content or replay charges
  External infrastructure owns the isolated candidate, identity gateway, synthetic
  provider/MCP peer, structured log and OTLP journals, and foreign-recorder lane.
  Scenarios never launch services, manage authentication, or spend on real models.
  Global/offline/shutdown fixtures remain tests/telemetry_ownership.rs; a five-second
  SDK acknowledgement does not establish hard exporter release or zero span loss.

  Background:
    Given the externally provisioned telemetry journals and isolated synthetic agent

  Scenario Outline: Actual retries and tools remain attributable through terminal observation
    When the mounted synthetic telemetry run reaches "<terminal>" after a retry and tool
    Then raw log and OTLP exports correlate its run request attempt and tool without private payloads
    And distinct resource IDs and queries share one bounded request metric route
    And read-only event replay preserves durable call accounting peer effects and terminal cost observations
    Examples:
      | terminal  |
      | done      |
      | cancelled |

  Scenario: An externally preinstalled host recorder is explicit at the mounted scrape
    Then the owned scrape contains activity and the foreign-recorder scrape reports host ownership
