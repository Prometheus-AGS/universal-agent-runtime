@api @providers @failure
Feature: Provider failure produces an honest recoverable result
  Scenario: A mid-stream provider disconnect fails honestly, then a fresh request succeeds
    Given the configured Liter provider is routed through an owned fault proxy
    When the provider connection is cut mid-stream during an admitted request
    Then the failed run never reports success or a fabricated answer
    And its model call is settled or explicitly uncertain without a model switch
    When the fault is removed and I send a fresh request
    Then the fresh run succeeds on the same configured model
