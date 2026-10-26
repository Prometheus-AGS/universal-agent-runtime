@api @streaming
Feature: Receive a streamed answer from the default agent
  As a user
  I want the agent's answer to arrive as incremental content
  So that I can read it before the response finishes

  Scenario: The default agent streams a complete useful answer
    Given UAR is running with a configured real model
    When I ask the default agent to count from 1 to 20 using streaming
    Then I receive multiple content events before the stream finishes
    And the assembled answer contains the numbers 1 through 20 in order
    And the stream ends successfully
