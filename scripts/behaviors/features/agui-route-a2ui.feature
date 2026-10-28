@api @agui @a2ui
Feature: An AG-UI app shows the agent's A2UI surface and the user's clicks reach the agent
  Scenario: The app shows the agent's surface, the user clicks it twice, and a reconnect still shows it
    Given a disposable agent on the dedicated AG-UI route that publishes a ticket surface with a note field and a Confirm button
    And a PEM 4.3.0 headless client configured with UAR's catalog ids
    When the client asks for a fresh ticket surface
    Then the client renders the surface with the requested heading and a Confirm button
    When the user enters a note and presses Confirm
    Then the agent answers in a new run on the same conversation and its answer contains that note
    When the user enters a different note and presses Confirm again
    Then the agent answers again and its answer contains the second note
    When a fresh client attaches to the conversation's latest run
    Then that client renders the same surface without the agent running again
    And an action naming a button the surface never declared is refused
