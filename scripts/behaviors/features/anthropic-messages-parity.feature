Feature: The Anthropic Messages endpoint runs the same agent turn as chat completions

  /v1/messages and /v1/chat/completions both run the selected agent; only the wire
  format differs (#396). The keyless stub model makes both answers deterministic.

  Scenario: A text answer and its usage match across both endpoints
    Given a dedicated UAR with the stub model
    When I ask the default agent through chat completions
    And I ask the default agent through Messages
    Then both endpoints return the same answer
    And both endpoints agree that the model reported no token usage

  Scenario: A caller tool call matches across both endpoints
    Given a dedicated UAR with the stub model
    When I ask the default agent to use my lookup_code tool through chat completions
    And I ask the default agent to use my lookup_code tool through Messages
    Then chat completions returns a lookup_code tool call
    And Messages stops with tool_use and a lookup_code block with the same input

  @messages-usage
  Scenario Outline: Controlled Messages and chat preserve known usage and ledger parity
    Given the externally provisioned owned Messages usage fixture
    When equivalent chat and Messages turns use "<mode>" provider usage with streaming "<stream>"
    Then both wire formats preserve the applicable usage and one settlement per call
    Examples:
      | mode    | stream |
      | nonzero | false  |
      | nonzero | true   |
      | zero    | false  |
      | zero    | true   |
      | absent  | false  |
      | absent  | true   |
      | uncached-only | false |
      | uncached-only | true  |

  @messages-usage
  Scenario Outline: A genuine two-call native tool loop never presents a partial sum as whole-run usage
    Given the externally provisioned owned Messages usage fixture
    When one real tool-loop run through each protocol uses "<mode>" usage with streaming "<stream>"
    Then whole-run usage is complete only when every submitted call reports it and original call liabilities remain
    Examples:
      | mode                  | stream |
      | loop-missing-known    | false  |
      | loop-missing-known    | true   |
      | loop-known-missing    | false  |
      | loop-known-missing    | true   |
      | loop-known            | false  |
      | loop-known            | true   |
      | loop-zero             | false  |
      | loop-zero             | true   |
      | loop-missing-read     | false  |
      | loop-missing-read     | true   |
      | loop-missing-creation | false  |
      | loop-missing-creation | true   |

  @messages-usage
  Scenario Outline: Messages cancellation terminates the real provider without late success
    Given the externally provisioned owned Messages usage fixture
    When I cancel an owned Messages turn at "<stage>"
    Then Messages ends with cancellation failure and the owned producer is closed
    Examples:
      | stage         |
      | before-output |
      | mid-output    |
      | tool-handoff  |

  @messages-usage
  Scenario: Messages caller-result replay preserves the exact owner batch and accounting
    Given the externally provisioned owned Messages usage fixture
    When an owned caller-tool result is continued and replayed through Messages
    Then replay and the foreign owner cannot create another provider call or settlement

  @messages-usage
  Scenario Outline: Unsupported Messages image forms refuse before conversion or dispatch
    Given the externally provisioned owned Messages usage fixture
    When I submit the unsupported Messages image form "<variant>"
    Then the Messages image request refuses explicitly without a new provider arrival
    Examples:
      | variant          |
      | missing-source   |
      | missing-data     |
      | remote-url       |
      | unsupported-mime |
      | invalid-base64   |
      | extra-source     |
      | unknown-block    |
      | assistant-image  |
      | system-image     |
      | tool-result-image |
