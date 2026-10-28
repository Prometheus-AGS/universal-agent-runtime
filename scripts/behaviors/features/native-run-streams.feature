@api @streaming @agui
Feature: Native run admission with legacy and official AG-UI streams
  Scenario: A native run's default stream delivers legacy AG-UI text and continues the session
    Given a disposable saved agent with a private instruction marker for native runs
    When I admit a fresh calculation and then a dependent follow-up as native runs and subscribe to each default stream
    Then each legacy run stream delivers incremental text, completes, and answers correctly in one session

  Scenario: A native run's agui_spec stream delivers official AG-UI events and continues the session
    Given a disposable saved agent with a private instruction marker for native runs
    When I admit a fresh calculation and then a dependent follow-up as native runs and subscribe to each agui_spec stream
    Then each official run stream starts and finishes its run once with correct answers in one session
