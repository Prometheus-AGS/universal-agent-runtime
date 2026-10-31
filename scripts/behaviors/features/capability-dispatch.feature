@api @capability-dispatch
Feature: Admitted images and constrained actual destinations on the owned Linux container
  The controlled peer proves typed wire delivery and destination admission.
  Its canned answer does not establish visual understanding.

  Background:
    Given the owned capability-dispatch fixture manifest and external identity endpoint

  @CD-01
  Scenario Outline: Mixed PNG text JPEG retains identity and order
    Given a disposable capability agent using "capable" with no fallback
    When I send "mixed" content to "<alias>" with streaming "<stream>"
    Then the actual capable destination receives the exact ordered parts
    And swapping the image hashes fails the same actual-request comparator
    Examples:
      | alias                | stream |
      | /v1/chat/completions  | false  |
      | /v1/chat/completions  | true   |
      | /api/chat/completion | false  |
      | /api/chat/completion | true   |

  @CD-02
  Scenario Outline: Literal text and image-only turns remain useful
    Given a disposable capability agent using "capable" with no fallback
    When I send "<kind>" content to "<alias>" with streaming "false"
    Then the actual capable destination receives the exact ordered parts
    Examples:
      | kind         | alias                |
      | image-only   | /v1/chat/completions  |
      | image-only   | /api/chat/completion |
      | text         | /v1/chat/completions  |
      | text-parts   | /api/chat/completion |
      | literal-json | /v1/chat/completions  |

  @CD-02
  Scenario: Text attachment projection precedes literal JSON-looking text
    Given a disposable capability agent using "capable" with no fallback
    When I send a text attachment before the literal current turn
    Then the actual capable destination receives the exact ordered parts

  @CD-02
  Scenario Outline: Text-only convenience message precedence remains compatible
    Given a disposable capability agent using "capable" with no fallback
    When I submit a "<convenience>" convenience message with text-only messages to "<alias>"
    Then the actual capable destination receives the exact ordered parts
    Examples:
      | convenience | alias                |
      | nonblank    | /v1/chat/completions  |
      | blank       | /v1/chat/completions  |
      | nonblank    | /api/chat/completion |
      | blank       | /api/chat/completion |

  @CD-03 @CD-10
  Scenario Outline: Invalid input refuses before any provider send
    Given a disposable capability agent using "capable" with no fallback
    And a valid control has reached that actual peer
    When I submit the invalid admission variant "<variant>"
    Then the public operation refuses and every owned peer has zero new sends
    And diagnostic sentinels are excluded by a sensitive privacy comparator
    Examples:
      | variant             |
      | remote-url          |
      | relative-url        |
      | file-url            |
      | upload-id           |
      | mime                |
      | base64              |
      | base64-padding      |
      | base64-url-alphabet |
      | base64-whitespace   |
      | empty-base64        |
      | mime-signature      |
      | header-only         |
      | decoded-bounds      |
      | detail              |
      | unknown-part        |
      | excess-image-count  |
      | envelope-bounds     |
      | image-attachment    |
      | ambiguous-message   |
      | earlier-image       |

  @CD-04
  Scenario Outline: Explicit incapable targets cannot receive hard demands
    Given a disposable capability agent using "<model>" with no fallback
    When I request the hard capability "<demand>" from that exact target
    Then the public operation refuses and every owned peer has zero new sends
    Examples:
      | model         | demand     |
      | text_only     | vision     |
      | no_tools      | tools      |
      | no_structured | structured |
      | tiny          | context    |

  @CD-04
  Scenario Outline: Actual capable destination receives declared hard controls
    Given a disposable capability agent using "capable" with no fallback
    When I request the hard capability "<demand>" from that exact target
    Then the actual destination receives the hard control "<demand>"
    Examples:
      | demand     |
      | tools      |
      | structured |
      | vision     |

  @CD-05
  Scenario: A registered disabled destination cannot reenter through preference
    Given a disposable capability agent using "disabled" with no fallback
    When I request the hard capability "vision" from that exact target
    Then the public operation refuses and every owned peer has zero new sends

  @CD-05
  Scenario: A real failure puts only the owned dedicated provider in cooldown
    Given a disposable capability agent using "cooldown" with no fallback
    When actual controlled failures reach the configured cooldown threshold
    Then the next exact attempt refuses with no new peer sends

  @CD-06
  Scenario: Permitted capable fallback recompiles its own actual request
    Given a disposable capability agent using "failure_primary" with fallback "fallback"
    And the real global outside fallback serves an unrestricted control
    When the controlled primary fails before output on a mixed image turn
    Then only the captured capable fallback succeeds with its own settings and images

  @CD-05 @CD-06
  Scenario Outline: Ineligible or exact-target fallback never sends
    Given a disposable capability agent using "<primary>" with fallback "<fallback>"
    And the real global outside fallback serves an unrestricted control
    When the primary attempt is "<selection>" on a mixed image turn
    Then the request fails without sending to the forbidden fallback or outside peer
    Examples:
      | primary       | fallback  | selection |
      | failure_exact | fallback  | exact     |
      | failure_scope | text_only | policy    |

  @CD-07
  Scenario: A later text-only request retains images without client replay
    Given a disposable capability agent using "capable" with no fallback
    When an image turn is followed by fresh text in the same owned session
    Then the second actual request retains the earlier ordered image hashes

  @CD-09
  Scenario: Finite financial liability refuses an unproven image bound before send
    Given a finite-cost disposable capability agent can send an ordinary text control
    When I send an admitted image under that same finite-cost policy
    Then the public operation refuses and every owned peer has zero new sends

  @CD-09 @CD-10
  Scenario: Estimated image delivery never advertises a fit guarantee
    Given a disposable capability agent using "capable" with no fallback
    When I send "mixed" content to "/v1/chat/completions" with streaming "true"
    Then the actual capable destination receives the exact ordered parts
    And the owned run events report estimated image accounting without a fit guarantee
    And diagnostic sentinels are excluded by a sensitive privacy comparator
