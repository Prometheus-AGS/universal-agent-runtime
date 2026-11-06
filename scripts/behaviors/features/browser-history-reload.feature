@ui @conversation @reload
Feature: A browser conversation reloads from saved history
  Scenario: Reopen a saved conversation after reload and continue from its earlier fact
    Given a disposable custom agent is selected in a fresh browser conversation
    When I give the agent a fresh fact in the browser
    And I reload the page and reopen that conversation from the thread list
    Then the reopened conversation shows the saved turns without a new run
    When I send a follow-up that contains none of the earlier facts
    Then the follow-up answer uses the earlier fact from the saved server history

  Scenario: A lost admission response reconciles its original turn after reload
    Given a disposable custom agent is selected in a fresh browser conversation
    When the server admits my browser turn but its response is lost
    And I reload after losing that admission response
    Then the original turn receipt and history recover without a second completion POST


  Scenario Outline: Reopen a controlled tool-loop conversation with canonical assistant segments
    Given the externally provisioned governed tool agent is selected in a fresh browser conversation
    When I complete a controlled "<intermediate>" tool-loop and a later browser turn
    And I reopen that tool-loop conversation with "<cache>" browser history
    Then every canonical assistant segment and final answer appears once in order without another run or effect
    When I reopen that tool-loop conversation with "warm-cache" browser history
    Then every canonical assistant segment and final answer appears once in order without another run or effect

    Examples:
      | intermediate | cache         |
      | preamble     | warm-cache    |
      | tool-only    | warm-cache    |
      | preamble     | fresh-context |
      | tool-only    | fresh-context |
