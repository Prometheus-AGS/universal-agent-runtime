@ui @chat
Feature: Chat surface anatomy (KnowMe idioms)
  As a user chatting with an agent
  I want the composer and message bubbles to follow the KnowMe Flat 2.0 idioms
  So that the conversation reads as authored content, not bordered form chrome

  Scenario: Composer is a filled surface without outlines
    Given a fresh conversation
    Then the composer shell has no visible border and no shadow
    And I capture a screenshot named "chat-composer-surface"

  Scenario: A user message renders as a trailing ember-tinted bubble
    Given a fresh conversation
    When I send the message "Visual evidence for the user bubble anatomy."
    Then the user bubble is trailing-aligned on an ember-tinted surface
    And I capture a screenshot named "chat-user-bubble"
