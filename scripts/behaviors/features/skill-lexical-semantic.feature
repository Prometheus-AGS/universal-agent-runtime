@api @skills @semantic
Feature: One prompt discriminates lexical from semantic selection
  Scenario: The keyword and local-embedding matchers pick different designed winners for the same prompt
    Given a disposable agent restricted to a lexical decoy skill and a semantic target skill
    When I send the discriminating prompt under keyword matching
    Then keyword matching selects the lexical decoy and the answer uses only the decoy's private rule
    When I send the identical prompt under local-embedding matching
    Then semantic matching selects the target and the answer uses only the target's private rule
