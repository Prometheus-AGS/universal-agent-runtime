# Spec Delta

## ADDED Requirements

### Requirement: Patch release claims identify accepted immutable artifacts

A Kubo integration patch release SHALL identify its source commit, immutable tag, publishing workflow commit/run, multi-platform digest and arm64/amd64 child digests. Release acceptance SHALL exercise the pulled published artifacts through production entry points without source patches and distinguish product acceptance from deployment health and customer recovery.

#### Scenario: Both published architectures pass
- **WHEN** v1.1.1 is published and the selected phase issue-repair integration program succeeds locally against both published child digests
- **THEN** the release includes the commands, source and fixture identity, image identities, architecture execution mode, scenario results and immutable receipt hashes
- **AND** native execution and emulation are labelled separately
- **AND** the release does not claim actual customer-cluster recovery from isolated fixture results

#### Scenario: Published architecture fails or is unavailable
- **WHEN** either required architecture program fails, cannot run or lacks a required observable
- **THEN** the release success claim is withheld and that gate is recorded as failed or blocked
- **AND** the published tag is not moved and passing historical v1.1.0 evidence is not substituted

#### Scenario: Local candidate passes before publication
- **WHEN** the production-path candidate image passes the local program before tag publication
- **THEN** its receipt identifies the local image and tested source tree
- **AND** it does not count as acceptance of the subsequently published digests

### Requirement: Release acceptance preserves the complete integration contract

The v1.1.1 runtime SHALL retain every accepted Kubo conformance behavior. Under the operator’s Execute instruction, the acceptance program SHALL cover only this phase’s specified binding, streaming, MCP outcome, credential and upgrade behaviors through real integration. A source-only test, helper assertion, skipped backend or smoke test SHALL NOT substitute for a required installed-artifact observation.

#### Scenario: Phase integration scope is bounded
- **WHEN** candidate and published image programs execute
- **THEN** the six phase groups cover binding, streams, MCP outcomes, credentials, recovery and combined behavior through real runtime and collaborators
- **AND** broader historical Kubo, defaults/egress and unrelated validation sweeps are not required by this phase's acceptance
- **AND** historical v1.1.0 receipts are retained without claiming they certify changed source

#### Scenario: New upgrade configuration differs from historical fixture
- **WHEN** the program exercises a budgeted run with an unattested remote store and then an independently qualified durable profile
- **THEN** it observes actionable refusal without provider/tool effects for the former and durable accounted execution/restart for the latter
- **AND** it does not silently add attestation to the refusal case

#### Scenario: Full-phase boundary is closed
- **WHEN** any planned production behavior, migration, fixture, configuration or required document remains incomplete
- **THEN** product verification and isolated artifact review remain deferred, including checks indirectly invoked by build or packaging scripts
- **AND** acceptance begins only after the complete inventory is explicitly recorded

#### Scenario: Isolated Docker network is unavailable before candidate startup
- **WHEN** the phase runner cannot create its owned Docker network before the image build or any scenario begins
- **THEN** the receipt records a blocked infrastructure prerequisite with no started group and retains the failed command log
- **AND** cleanup does not claim failure to remove a network that was never created

### Requirement: Issue disposition separates delivered scope from external dependencies

Release handoff SHALL map #379, #380, #382, #383 and #385 to their specified criteria and evidence. Unverified deployment facts, external ownership and cancelled qualification SHALL remain explicit; issue closure SHALL NOT imply unsupported downstream integration or infrastructure recovery.

#### Scenario: Previously delivered requests are reconciled
- **WHEN** #379 and #383 are reconciled
- **THEN** the handoff links their immutable v1.1.0 release and arm64 plus emulated-amd64 receipts and identifies the later #385 repair separately
- **AND** v1.1.0 history remains unchanged

#### Scenario: Customer recovery evidence is absent
- **WHEN** isolated #385 acceptance passes but the live persisted agent, durable backing or authorized recovery result is unavailable
- **THEN** product repair is recorded separately and actual customer recovery remains unverified
- **AND** a customer-recovery criterion is not closed as passed

#### Scenario: External Dot dependency remains open
- **WHEN** #382 depends on work owned by other repositories or an unqualified integration lane
- **THEN** UAR records its delivered dependency status and the external owner/blocker
- **AND** the tracker remains open and the cancelled Dot qualification program is not restarted
