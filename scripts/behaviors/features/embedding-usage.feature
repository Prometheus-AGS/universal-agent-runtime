@api @embedding @usage
Feature: Mounted embeddings retain reported usage and backend identity
  The external host owns an isolated candidate and keyless synthetic embedding
  peers. Functional scenarios carry no credentials or authentication setup.

  Background:
    Given the externally provisioned owned embedding usage fixture

  Scenario Outline: Reported counters remain independent through the mounted route
    When the owned embedding peer reports "<mode>"
    And I request the configured embedding model using "<encoding>"
    Then embedding vectors and usage match the owned "<mode>" oracle
    Examples:
      | mode        | encoding |
      | nonzero     | float    |
      | nonzero     | base64   |
      | zero        | float    |
      | absent      | float    |
      | prompt-only | float    |
      | total-only  | float    |

  Scenario Outline: Malformed provider results refuse the complete answer
    When the owned embedding peer reports "<mode>"
    And I request the configured embedding model using "float"
    Then the mounted embedding operation refuses the provider result privately
    Examples:
      | mode               |
      | malformed-usage    |
      | duplicate-index    |
      | missing-index      |
      | out-of-range-index |
      | incomplete         |
      | bad-dimension      |
      | provider-error     |

  Scenario: Both served model forms preserve exact dimensions
    When the owned embedding peer reports "nonzero"
    Then both served embedding model names return exact dimensions

  Scenario Outline: Invalid public requests never invoke the configured backend
    When the owned embedding peer reports "nonzero"
    Then the embedding request variant "<variant>" refuses before any backend arrival
    Examples:
      | variant            |
      | unknown-model      |
      | wrong-dimensions   |
      | empty-input        |
      | empty-string       |
      | oversized-batch    |
      | unsupported-format |
      | oversized-body     |

  Scenario Outline: External admission rejection never invokes the backend
    When the owned embedding peer reports "nonzero"
    Then the externally provisioned "<lane>" embedding ingress refuses before a backend arrival
    Examples:
      | lane             |
      | missing-identity |
      | invalid-identity |
      | denied-policy    |
      | rate-limited     |
      | unavailable      |

  Scenario: A settings swap preserves the admitted embedding space and accounting
    When an owned embedding request is held while its configured backend is swapped
    Then the admitted request retains its original vectors model and usage
    And the next embedding request uses the new configured space and usage
