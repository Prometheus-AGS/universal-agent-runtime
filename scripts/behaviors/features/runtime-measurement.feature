@api @measurement @synthetic
Feature: A selected deterministic packaged-runtime baseline retains measurements and limits
  There is no default workload, concurrency, hardware profile or SLO.
  A complete owner selection is required before any HTTP request is issued.
  External infrastructure owns identity, startup, resource observations and peers.
  This feature never launches services, probes processes, handles credentials or pays for inference.

  Scenario: Measure exactly the frozen synthetic workload and selected promises
    Given a complete owner-selected packaged-runtime measurement manifest
    When the selected streaming workload is measured at its declared concurrency and repetitions
    Then raw measurements retain semantic latencies provider time startup resource and reliability availability
    And only selected thresholds with available evidence may qualify
