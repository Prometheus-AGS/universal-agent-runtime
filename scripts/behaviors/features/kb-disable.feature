@api @kb-disable
Feature: Disable agent knowledge access without removing saved links
  Scenario: A disabled agent cannot use its still-linked private knowledge
    Given a disposable agent with enabled KB access and a saved private-fact link
    And the enabled agent uses the private fact with a matching knowledge citation
    When I disable KB access without changing the saved links
    Then the disabled agent answers without KB knowledge while its linked source remains searchable
