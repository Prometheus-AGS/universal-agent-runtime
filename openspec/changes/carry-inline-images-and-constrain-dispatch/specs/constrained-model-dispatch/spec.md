## Purpose
Enforce hard payload and captured destination constraints on actual model sends using the existing routing and budget facilities. Advisory metadata alone does not certify execution capability.

## ADDED Requirements

### Requirement: Shared candidate admission
Advisory routing and actual dispatch SHALL use one explained capability/availability contract. Explicit registered disablement SHALL veto catalog supplementation. Preferences SHALL not override hard request requirements, cooldown or captured destination scope; missing required facts SHALL not be assumed certified.

#### Scenario: Preferred model is ineligible
- **WHEN** the preferred destination is disabled, cooling down or fails an actual tools/vision/structured/context requirement
- **THEN** it is not sent solely due to preference or catalog re-entry

### Requirement: Final-send scope enforcement
Every public primary, retry, fallback and graph send SHALL derive requirements from its final payload and stay within immutable actual captured model identities. Exact targets SHALL not substitute. Admission rejection SHALL not manufacture fallback authority or be treated as a provider outage.

#### Scenario: No eligible destination
- **WHEN** all captured candidates violate hard requirements or authority
- **THEN** execution explicitly refuses with no provider call

#### Scenario: Eligible fallback
- **WHEN** a permitted pre-output primary failure permits fallback
- **THEN** only a capable captured fallback is sent after fresh admission and its own destination preparation
- **AND** excluded or incapable fallback destinations receive zero calls

### Requirement: Preserve final-destination budget honesty
Dispatch SHALL preserve existing destination-bound endpoint/model/settings, output/reasoning reserves and protected groups. Unproven real-image accounting SHALL not claim guaranteed fit. Strict fit or finite accounting requiring an unproven image count SHALL refuse before dispatch; estimated lanes SHALL state their limit.

#### Scenario: Image count contract is unproven
- **WHEN** a real image request requires guaranteed fit without validated image-aware counting
- **THEN** it refuses before provider dispatch instead of substituting a flat allowance or JSON count as real image accounting

#### Scenario: Synthetic exact protected overflow
- **WHEN** an explicitly synthetic exact wire-count contract cannot fit the protected image/tool group and output/reasoning reserves
- **THEN** it refuses before send and the receipt credits only that synthetic contract

### Requirement: Actual execution evidence
Acceptance SHALL exercise actual mounted public runs and actual provider-adapter destinations with at least two deterministic registrations and controlled fallback. Route-query success, catalog declarations or compilation alone SHALL not establish dispatch behavior or provider quality.

#### Scenario: Capability fixture selects real send
- **WHEN** mounted public cases require tools, vision, structured output or bounded context
- **THEN** captured sends and zero-send controls prove only eligible destinations were used
- **AND** receipts retain exact source/config/artifact scope and distinguish real-model qualification

### Requirement: Captured final-wire reservation
Durable model reservations under an enforceable request contract SHALL obtain their input bound and effective output ceiling from the captured driver's actual preparation after host output narrowing and before provider dispatch. Existing uncertainty, reasoning, atomic admission, ownership and uncertain-settlement rules SHALL remain. UAR preflight serialization SHALL not substitute for unavailable final preparation. Serialized image JSON SHALL not certify real-image accounting. Existing no-contract and no-durable-reservation paths SHALL not acquire an unrelated preparation prerequisite.

#### Scenario: Routing identity differs from literal wire model
- **WHEN** retained adapter preparation strips a qualified routing prefix
- **THEN** final validation, counting and receipts use the actual prepared body while preserving captured destination, declared profile and protected request controls
- **AND** routing identity remains distinct from the literal provider-body model

#### Scenario: Durable contract lacks prepared evidence
- **WHEN** a durable contracted driver cannot supply supported captured final preparation
- **THEN** admission refuses before model dispatch instead of recording preflight as final-wire evidence

#### Scenario: Existing uncontracted path
- **WHEN** no request contract or no durable reservation requires final preparation
- **THEN** the existing catalog, team or no-reservation admission path remains in force


### Requirement: Preserve endpoint profiles without finite-budget contracts

An installed endpoint profile SHALL preserve its declared literal model, allowed fields and typed controls in the actual provider-prepared body even when no finite-budget contract is installed. The absence of a budget contract SHALL omit budget arithmetic, not final profile validation. Ordinary unprofiled and uncontracted requests SHALL retain their existing path.

#### Scenario: Adapter changes an uncontracted profile literal
- **GIVEN** a settings-only profile with no output ceiling and no budget contract
- **WHEN** the captured adapter preparation changes the declared literal wire model
- **THEN** dispatch refuses before a provider stream opens

#### Scenario: Adapter preserves an uncontracted profile
- **GIVEN** a settings-only profile with no output ceiling and no budget contract
- **WHEN** the actual prepared model, schema and typed controls match the captured profile
- **THEN** final profile validation succeeds without creating a budget counting contract


### Requirement: Fit fallback history against its effective output cap

Estimated fallback fitting SHALL apply the captured destination's downward output restriction before reserving output or displacing optional history. A higher primary output cap SHALL NOT reject an otherwise viable fallback. A caller's lower output cap SHALL NOT be raised. Exact destination preparation SHALL retain its own contract and canonical-history behavior. Fresh admission of the final prepared request SHALL remain immediately before dispatch. Fit diagnostics SHALL describe the effective reserve and retain estimated counting and no guaranteed-fit claim.

#### Scenario: Fallback has a lower output maximum
- **GIVEN** a permitted fallback with an 8192-token window and 2048-token maximum output, a short prompt and a primary output cap of 16384
- **WHEN** an authorized pre-output primary failure invokes fallback
- **THEN** the request is fitted using the fallback's downward cap and can dispatch with output at most 2048 after fresh final admission
- **AND** the earlier primary cap does not cause overflow refusal or unnecessary history displacement

#### Scenario: Caller cap is already lower
- **GIVEN** a caller output cap below the fallback maximum
- **WHEN** that permitted fallback is fitted
- **THEN** the lower caller cap remains and the fit diagnostics use that effective reserve
