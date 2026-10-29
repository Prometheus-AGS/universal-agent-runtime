@api @tools @approval @native
Feature: Approved native tool results reach the answer; denial has no effect
  Scenario: Approve one native file write and deny another
    Given native file tools are enabled for one owned directory after an owned restart
    And a disposable agent that may use only the native file_write tool
    When I ask the agent to write a private note and approve the file_write request
    Then the approved note exists exactly once on disk and the answer reports the tool's real result
    When I ask the agent to write a second note and deny the file_write request
    Then the second note does not exist and the answer reports that the write was not performed
