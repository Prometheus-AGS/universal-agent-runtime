@artifact-qualification
Feature: Exact owned Linux container and independent protocol qualification
  Host infrastructure owns identity, container transitions and data backup operations.
  Each explicitly selected stage records observations; no stage claims the others passed.

  @documents
  Scenario: Advertised document families survive upload extraction indexing and retrieval
    Given the exact owned Linux API artifact is externally provisioned
    When I ingest the five tiny qualification document formats
    Then each format retrieves its own fact with document provenance
    And a malformed Office input is refused without a searchable fact

  @capture-prior
  Scenario: Capture actual prior artifact and synthetic retained state
    Given the exact owned Linux API artifact is externally provisioned
    And the clean release candidate identity is bound for measurement
    And the owned controlled streaming model is externally provisioned
    And a frozen bounded reliability agent is retained for this case
    When I save the artifact lifecycle canary
    Then the retained canary and exact prior artifact are recorded

  @observe-stopped
  Scenario: Observe an externally stopped owned artifact after drain
    Given the owned Linux container is stopped externally
    Then its observed clean exit and retained mount are recorded

  @observe-restart
  Scenario: Observe real same-artifact restart and retained state
    Given the exact owned Linux API artifact is externally provisioned
    And the owned controlled streaming model is externally provisioned
    Then the same artifact restarted after the captured clean stop and retains the canary

  @observe-upgrade
  Scenario: Observe actual prior-to-current installed upgrade on the retained volume
    Given the exact owned Linux API artifact is externally provisioned
    Then a distinct current artifact retains the prior canary and volume

  @delete-after-backup
  Scenario: Delete owned canary after an externally produced stopped-store backup
    Given the exact owned Linux API artifact is externally provisioned
    When I bind the owned backup and delete the lifecycle canary
    Then the deleted canary is absent before restore

  @observe-restore
  Scenario: Observe recovery of deleted data from the bound backup
    Given the exact owned Linux API artifact is externally provisioned
    Then the bound backup is unchanged and the deleted canary has returned

  @observe-rollback
  Scenario: Observe explicit rollback to the exact prior image
    Given the exact owned Linux API artifact is externally provisioned
    Then the original artifact retains the restored canary after rollback

  @independent-a2a
  Scenario: Official independently pinned A2A client observes task completion and refusal
    Given the exact owned Linux API artifact is externally provisioned
    When the official pinned A2A client uses the advertised JSONRPC streaming interface
    Then it observes a completed task and unsupported A2A methods fail explicitly

  @independent-mcp
  Scenario: UAR uses an official independently pinned MCP server behind tool approval
    Given the exact owned Linux API artifact is externally provisioned
    And the clean release candidate identity is bound for measurement
    And the owned controlled streaming model is externally provisioned
    And the official MCP peer and scenario-scoped tool policy are externally provisioned
    When I admit one MCP echo and deny another through native run tool approvals
    Then the peer journal contains exactly the admitted marker
    When an approved independent MCP request is pending and I cancel its native owner
    Then the same SDK request terminates without completion or an effect claim
    When an approved independent MCP request returns an intentional tool error
    Then the actual tool result is an error and the peer remains usable

  @independent-a2a-cancellation
  Scenario: Official A2A client cancels an active original task and decodes task errors
    Given the exact owned Linux API artifact is externally provisioned
    And the owned controlled streaming model is externally provisioned
    When the official A2A SDK cancels an actually active original task
    Then the independent task remains canceled and original missing or terminal task errors are explicit
    And a new independent A2A task completes after cancellation

  @active-cancellation
  Scenario: Cancel actual active execution and retain truthful terminal accounting
    Given the exact owned Linux API artifact is externally provisioned
    And the clean release candidate identity is bound for measurement
    And the owned controlled streaming model is externally provisioned
    When a controlled native run actually begins streaming and I cancel it
    Then cancellation closes the provider stream without later success or hidden liabilities
    And a new controlled native run still completes after cancellation

  @interrupted-stream
  Scenario: An actual interrupted provider stream errors and a fresh run still completes
    Given the exact owned Linux API artifact is externally provisioned
    And the clean release candidate identity is bound for measurement
    And the owned controlled streaming model is externally provisioned
    And a frozen bounded reliability agent is retained for this case
    When the controlled provider cuts a stream after its first semantic chunk
    Then the interrupted run records error without successful completion
    And a fresh run under the same frozen profile completes

  @long-conversation
  Scenario: A bounded conversation retains actual earlier inputs in later provider requests
    Given the exact owned Linux API artifact is externally provisioned
    And the clean release candidate identity is bound for measurement
    And the owned controlled streaming model is externally provisioned
    And a frozen bounded reliability agent is retained for this case
    When sixteen sequential turns use the same retained conversation
    Then canonical history and later provider requests retain the original inputs

  @admission-races
  Scenario: Concurrent exact admission and an abandoned response keep one budgeted run
    Given the exact owned Linux API artifact is externally provisioned
    And the owned controlled streaming model is externally provisioned
    When two concurrent identical full-harness admissions race on one bounded request
    Then their stable identities and retained completion refer to one execution
    And an abandoned admission response is reconciled by identity without redispatch

  @remote-surreal-capture
  Scenario: Capture an explicitly selected Linux remote-Surreal profile before outage
    Given the exact owned Linux API artifact is externally provisioned
    And the clean release candidate identity is bound for measurement
    And the owned remote Surreal profile is externally provisioned
    When I retain the remote profile canary and original process

  @remote-surreal-down
  Scenario: Observe an externally stopped owned Surreal server while UAR stays alive
    Given the owned Linux API artifact remains running during an external database outage
    Then the remote profile reports transport loss and unavailable persistence

  @remote-surreal-recovered
  Scenario: Observe restored remote transport without replacing the Linux UAR process
    Given the exact owned Linux API artifact is externally provisioned
    And the owned remote Surreal profile is externally provisioned
    Then the same remote profile recovers the canary and accepts an owned write

  @capture-epoch
  Scenario: Capture an actual process-ephemeral full-harness admission before restart
    Given the exact owned Linux API artifact is externally provisioned
    And the owned controlled streaming model is externally provisioned
    When I capture a settled full-harness task and its runtime epoch
    Then the captured epoch fixture is retained for the external transition

  @observe-epoch
  Scenario: Old full-harness controls cannot acquire effects after actual process replacement
    Given the exact owned Linux API artifact is externally provisioned
    And the owned controlled streaming model is externally provisioned
    Then the old task cursor admission and controls refuse under the new epoch without dispatch

  @postgres-refusals
  Scenario: Restricted PG18 explicitly refuses each unsupported durable surface
    Given the exact owned Linux API artifact is externally provisioned
    Then PostgreSQL capabilities and all selected unsupported APIs agree

  @postgres-capture
  Scenario: Capture owner-scoped PG18 state on the actual pinned database volume
    Given the exact owned Linux API artifact is externally provisioned
    And the owned PG18 store and second identity endpoint are externally provisioned
    When I save a PG18 owner-scoped restore canary
    Then the other externally supplied owner cannot read that canary

  @postgres-delete-after-backup
  Scenario: Delete PG18 synthetic state after the external host binds its actual backup
    Given the exact owned Linux API artifact is externally provisioned
    And the owned PG18 store and second identity endpoint are externally provisioned
    When I bind the PG18 backup and delete the PG18 restore canary
    Then the PG18 canary is absent before external restore

  @postgres-observe-restore
  Scenario: Restored PG18 data retains ownership and the declared restricted profile
    Given the exact owned Linux API artifact is externally provisioned
    And the owned PG18 store and second identity endpoint are externally provisioned
    Then the PG18 backup is unchanged and restored data remains owner scoped
    And PostgreSQL capabilities and all selected unsupported APIs agree

  @embedded-surrealdb4-seed
  Scenario: A fresh embedded 4.0 artifact writes an owner-scoped canary
    Given the exact owned Linux API artifact is externally provisioned
    When I write an embedded upgrade owner canary
    Then the separately provisioned owner cannot read the upgrade canary

  @embedded-surrealdb4-stop
  Scenario: The embedded 4.0 artifact exits cleanly before reopen
    Given the owned Linux container is stopped externally
    Then the embedded upgrade artifact has stopped with its original volume

  @embedded-surrealdb4-reopen
  Scenario: The same embedded 4.0 artifact reopens its owned data
    Given the exact owned Linux API artifact is externally provisioned
    Then the embedded upgrade artifact reopens and retains the owner canary
