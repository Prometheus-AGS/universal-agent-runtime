@api @context @budget
Feature: Whole-request overflow refuses before an over-budget dispatch
  Scenario: Non-removable content larger than a small real allowance is refused
    Given a disposable provider route whose model has a small context window
    And a disposable agent with long required instructions, tools and a tiny linked knowledge base
    When I send a request whose required content exceeds that window
    Then a typed budget refusal is reported before any provider dispatch
    And no required content was silently dropped to make it fit
