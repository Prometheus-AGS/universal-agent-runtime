@api @memory @restart
Feature: Durable memory can be recalled, updated and removed
  Scenario: Save, recall, update across restart, then delete a private memory
    Given durable memory is enabled for the owned runtime after an owned restart
    And a disposable agent with conversation memory and no knowledge or tools
    When I save a private locker code to memory
    Then a fresh conversation recalls that locker code from memory
    When I update the saved locker code and UAR restarts
    Then a fresh conversation recalls only the updated locker code from memory
    When I delete the saved locker code
    Then a fresh conversation no longer knows the locker code
    And after another restart the deleted memory is still absent
