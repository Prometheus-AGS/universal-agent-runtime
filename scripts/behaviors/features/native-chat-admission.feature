@api @native-chat @durable-admission
Feature: Native chat admission preserves one original turn across transport loss
  Background:
    Given the externally provisioned native chat admission fixture

  Scenario Outline: Independent concurrent transports share one governed execution
    When two independent native chat POSTs carry the same original identity for "<mode>"
    Then both transports and a later identical POST reconcile the original run
    And the independent journals show exactly <arrivals> provider arrivals and <effects> committed record effects
    Examples:
      | mode    | arrivals | effects |
      | success | 1        | 0       |
      | effect  | 2        | 1       |

  Scenario Outline: Original identity cannot be repurposed
    When one native chat turn succeeds with its original identity
    And I reuse that turn identity with changed "<field>"
    Then the changed request conflicts without another provider arrival or effect
    Examples:
      | field       |
      | message     |
      | policy      |
      | attachments |

  Scenario: Discarded admission response is recovered without another POST
    When I discard the response of an admitted native chat turn
    Then read-only turn reconciliation retains its original run and one provider arrival

  Scenario: Rich original content and identity survive canonical history reads
    When a native chat turn with a rich image completes
    Then repeated canonical history reads retain the original rich user message and completed assistant

  Scenario Outline: Partial output does not become fabricated completion
    When a native chat "<mode>" turn produces partial output
    Then its receipt and canonical partial history retain "<state>" without a replacement run
    And the independent journals show exactly 1 provider arrivals and 0 committed record effects
    Examples:
      | mode          | state     |
      | partial-error | error     |
      | hold          | cancelled |

  Scenario: An answered approval remains historical after canonical reload
    When a native governed tool turn is approved through its originating approval
    Then repeated canonical history reads retain that exact approval decision without new authorization
    And the independent journals show exactly 2 provider arrivals and 1 committed record effects

  Scenario: External restart leaves interrupted partial history recoverable
    Given an externally captured interrupted native turn across a completed runtime restart
    Then read-only reconciliation retains interrupted partial history without any new provider arrival or effect

  Scenario: Caller-owned tool batches keep their distinct continuation identity
    When I attach native turn identity to a caller-owned tool batch
    Then native identity is explicitly unsupported before any provider arrival

  Scenario: Legacy native chat without the additive IDs still follows its original stream path
    When I send a legacy native chat turn without additive identities
    Then the legacy stream succeeds without claiming protected turn admission


  Scenario: Recovery precedes mutable agent resolution after a catalog deletion
    When one native turn completes under a per-case disposable copy of the configured agent
    And I delete only that confirmed per-case agent through the mounted catalog API
    Then the identical original request returns its prior receipt despite the deleted agent and never dispatches again
