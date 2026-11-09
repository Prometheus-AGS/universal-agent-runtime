@api @memory @controlled-provider
Feature: Independent memory inclusion is proved at the final provider boundary
  # External infrastructure provisions separate identity endpoints, stores and
  # peer before this credential-free suite. No step starts/restarts a service.
  Scenario: A fresh independent memory reaches the final System input with no history or memory tools
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "relevant" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "success"
    Then the exact saved fact appears only in the captured retrieved System content
    And the host reports the original memory IDs submitted with provider-response evidence
    And the native run completes without a tool or replayed history

  Scenario: An absent operator cutoff refuses automatic memory selection
    Given the externally provisioned memory-context fixture profile "absent"
    When I save a fresh "relevant" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "success"
    Then memory-context reports "refused" with reason "memory_relevance_not_configured"
    And no saved fact or submitted claim reaches the provider boundary

  Scenario: A real owner-visible hybrid candidate fails the configured cosine cutoff
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "below-cutoff" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "success"
    Then memory-context reports "no_match" with reason "memory_below_similarity_or_removed"
    And the independent memory engine compared opposite candidate and query vectors
    And no saved fact or submitted claim reaches the provider boundary

  Scenario: Per-turn opt-out prevents automatic injection
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "relevant" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "opt-out"
    Then memory-context reports "opt_out" with reason "memory_turn_opt_out"
    And no saved fact or submitted claim reaches the provider boundary

  Scenario: Deletion excludes a saved independent fact without a subsequent successful turn
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "relevant" memory through the independent memory API
    And I delete that exact independent memory
    And I submit a fresh memory-context turn in mode "success"
    Then memory-context reports "no_match" with reason "memory_no_retained_scope_match"
    And no saved fact or submitted claim reaches the provider boundary

  Scenario: A foreign credential identity cannot supply or read this owner's memory
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "foreign" memory through the independent memory API
    Then the primary identity cannot read the foreign record
    When I submit a fresh memory-context turn in mode "success"
    Then memory-context reports "no_match" with reason "memory_no_retained_scope_match"
    And no saved fact or submitted claim reaches the provider boundary

  Scenario: Final context refusal cannot manufacture a memory submission
    Given the externally provisioned memory-context fixture profile "budget"
    When I save a fresh "relevant" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "budget-refusal"
    Then the final context budget refuses the turn before the model peer receives it
    And no saved fact or submitted claim reaches the provider boundary

  Scenario: A first provider error does not prove submission despite a prepared request
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "relevant" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "error-first"
    Then the exact saved fact appears only in the captured retrieved System content
    And no submitted claim is published and the native run fails

  Scenario: Partial provider text proves input submission but does not prove completion
    Given the externally provisioned memory-context fixture profile "positive"
    When I save a fresh "relevant" memory through the independent memory API
    And I submit a fresh memory-context turn in mode "partial-error"
    Then the exact saved fact appears only in the captured retrieved System content
    And the host reports the original memory IDs submitted with provider-response evidence
    And partial provider text remains an error outcome rather than completed recall
