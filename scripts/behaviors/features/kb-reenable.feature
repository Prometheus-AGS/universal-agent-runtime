@api @kb-reenable
Feature: Re-enable agent knowledge access using retained links
  Scenario: Re-enabled access restores use of the original linked knowledge
    Given a disposable agent with disabled KB access and its original saved link
    When I re-enable KB access without resending the retained links
    Then a fresh answer uses the original retained knowledge with matching provenance
