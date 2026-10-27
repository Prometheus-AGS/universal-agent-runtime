@ui @conversation @reload
Feature: A browser conversation reloads from saved history
  Scenario: Reopen a saved conversation after reload and continue from its earlier fact
    Given a disposable custom agent is selected in a fresh browser conversation
    When I give the agent a fresh fact in the browser
    And I reload the page and reopen that conversation from the thread list
    Then the reopened conversation shows the saved turns without a new run
    When I send a follow-up that contains none of the earlier facts
    Then the follow-up answer uses the earlier fact from the saved server history
