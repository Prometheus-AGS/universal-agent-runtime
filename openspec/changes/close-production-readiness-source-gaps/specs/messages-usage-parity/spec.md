## ADDED Requirements

### Requirement: Preserve shared Messages accounting
Messages streaming/nonstreaming SHALL project applicable shared input/output/cache counters without charging again, and unsupported counters SHALL be explicitly unavailable. Cancellation/retry SHALL preserve owner/result/effect safeguards.

#### Scenario: The same completed run usage reaches the Chat stream
- **WHEN** the OpenAI-shaped chat stream reaches a successful stop with reported run usage
- **THEN** the existing stop chunk carries the same usage object as nonstreaming chat, with unchanged frame order and cursor ordinals
- **AND** a successful stop with unavailable usage omits the usage object, and neither projection charges again

#### Scenario: A controlled provider reports nonzero cached usage then interruption
- **WHEN** A controlled provider reports nonzero cached usage then interruption
- **THEN** Projection and ledger retain known counts and truthful terminal outcome, with no duplicate charge or expanded tool authority

#### Scenario: A submitted tool-loop call has no usage report
- **WHEN** A genuine multi-call run contains a submitted provider call without applicable usage and another call with measured usage, in either order
- **THEN** Messages and OpenAI aggregate projections do not present the partial sum as complete provider-reported run usage
- **AND** Known per-call reports, original call identities, accounting liabilities and settlement boundaries remain retained without zero substitution or duplicate charges

#### Scenario: Completeness is per counter and per submitted call
- **WHEN** All submitted calls report input/output but an applicable cache counter is missing from one call
- **THEN** Complete input/output may retain their exact aggregate while that cache aggregate remains unavailable
- **AND** All-reported and known-zero counters remain measured, repeated cumulative reports within one call count once, and an iteration refused before dispatch is not counted as a submitted provider call

#### Scenario: Native uncached input remains known when a cache aggregate is unavailable
- **WHEN** Every submitted native call reports uncached input but a cache breakdown counter is absent from one call
- **THEN** Explicit optional uncached-input evidence preserves the exact Messages sum independently of the complete raw OpenAI input sum
- **AND** An unsupported uncached counter is unavailable unless a complete checked cache split or measured raw zero proves it; unknown cache is never substituted as zero

#### Scenario: Registered provider resolution preserves native tool concurrency
- **WHEN** An enabled configured provider and model are resolved for an existing native tool route
- **THEN** The resolved client preserves the operator's explicit parallel-tool setting, including false, and retains the default when unset
- **AND** Destination capability, credentials and other admission checks remain in force
