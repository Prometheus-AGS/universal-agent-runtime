@api @knowledge @embedding @restart
Feature: Interrupted KB regeneration resumes honestly
  Scenario: Restart UAR mid-regeneration, then resume to one target generation
    Given a disposable multi-document knowledge base indexed with the original embedding model
    And the active embedding model is changed to a different model of the same width
    When regeneration starts and the owned UAR is restarted before it completes
    Then after the restart the job reports an honest unfinished state and the sources are retained
    When regeneration is resumed through its designed operation
    Then it completes in one target generation and retrieval returns a private fact
