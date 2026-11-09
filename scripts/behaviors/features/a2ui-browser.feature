@ui @a2ui
Feature: Interact with a real A2UI response in the browser
  Scenario: A rendered calculator sends fresh input back to its agent
    Given a disposable A2UI calculator agent is selected in the browser
    When I ask the agent to render its calculator
    Then the browser displays the generated input and action button
    When I enter a fresh number and press the rendered calculate button
    Then the agent answers using that browser input and its saved instructions

  Scenario: A lost logical action response recovers the original continuation
    Given a disposable A2UI calculator agent is selected in the browser
    When I ask the agent to render its calculator
    Then the browser displays the generated input and action button
    When the calculator action is admitted but its browser response is lost
    Then recovering that action reads its original receipt without another action POST

  Scenario: A newer publication cannot authorize an old rendered calculator
    Given a disposable A2UI calculator agent is selected in the browser
    When I ask the agent to render its calculator
    Then the browser displays the generated input and action button
    When the same source publishes a newer calculator while the original control remains displayed
    Then pressing the original calculator refuses the stale generation without an action POST
