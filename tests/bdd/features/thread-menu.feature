@ui @thread-menu
Feature: Thread row menu renames and deletes conversations
  As a person chatting
  I want each thread's row menu to rename or delete that conversation
  So that my thread list reads the way I named it and what I deleted stays gone

  # Repair round 4 of phase uar-uiux-architecture-2026-11 (SCN-THR-001, SCN-THR-002).
  # Authored, not executed. Each scenario has a negative control that cancels the dialog.
  # The menu is the row's "Actions for <title>" button in the sidebar thread list. The
  # reload steps show that the change reached the browser's local database. BDD runs the
  # embedded store, which sends no thread snapshot, so they cannot show whether a
  # postgres or remote-Surreal server snapshot re-adds a deleted thread or resets a title
  # (follow-ups P2 and D1). BDD runs use the deployment's auth-off test configuration and
  # send no credentials.

  @SCN-THR-001
  Scenario: Renaming a thread from its row menu keeps the new name after a reload
    Given a fresh conversation
    When I send the message "What is 2 plus 2?"
    And I rename this thread to "BDD Renamed Thread" through its row menu
    Then the renamed thread is titled "BDD Renamed Thread" and no longer shows its previous title
    And the thread list shows "BDD Renamed Thread" exactly once
    When I reload the page
    Then the thread list shows "BDD Renamed Thread" exactly once

  # Control of SCN-THR-001: cancelling the rename dialog changes nothing.
  @CTL-THR-001-CANCEL
  Scenario: Cancelling the rename dialog keeps the thread's title
    Given a fresh conversation
    When I send the message "What is 2 plus 2?"
    And I start renaming this thread to "BDD Cancelled Name" and cancel
    Then the thread still shows its previous title
    And the thread list does not show "BDD Cancelled Name"

  @SCN-THR-002
  Scenario: Deleting a thread from its row menu removes it for good and keeps the others
    Given a fresh conversation
    And I send the message "What is 2 plus 2?"
    And this thread is named "BDD Thread A"
    And I start a new thread
    And I send the message "What is 2 plus 2?"
    And this thread is named "BDD Thread B"
    When I choose Delete in the row menu of "BDD Thread A"
    Then the delete confirmation names "BDD Thread A"
    When I confirm the deletion
    Then the thread list shows "BDD Thread B"
    And the thread list does not show "BDD Thread A"
    When I reload the page
    Then the thread list shows "BDD Thread B"
    And the thread list does not show "BDD Thread A"

  # Control of SCN-THR-002: cancelling the confirmation deletes nothing.
  @CTL-THR-002-CANCEL
  Scenario: Cancelling the delete confirmation keeps the thread
    Given a fresh conversation
    And I send the message "What is 2 plus 2?"
    And this thread is named "BDD Thread Kept"
    When I choose Delete in the row menu of "BDD Thread Kept"
    And I cancel the deletion
    Then the thread list shows "BDD Thread Kept"
