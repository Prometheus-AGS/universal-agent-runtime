@api @kb-relink
Feature: Restore an agent's access to an existing knowledge base
  Scenario: Relinking an existing KB restores grounded answers
    Given a disposable agent whose original knowledge base link has been removed
    When I restore the original knowledge base link on that agent
    Then a fresh answer uses the original private fact with a matching citation
