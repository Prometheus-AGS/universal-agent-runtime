@api @two-kb-answer
Feature: Combine facts from two linked knowledge bases
  Scenario: An agent uses both KBs to answer a combined inventory question
    Given a disposable agent and two indexed knowledge bases with separate private facts
    When I link both knowledge bases to the agent
    And I ask for the two inventories and their combined total in a fresh conversation
    Then the answer combines both private facts with a matching citation for each knowledge base
