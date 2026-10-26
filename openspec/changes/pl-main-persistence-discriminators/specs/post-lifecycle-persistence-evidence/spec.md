## ADDED Requirements

### Requirement: Main persistence evidence uses real owned records and controls

History and knowledge-base discriminators SHALL exercise the selected main process and eligible isolated persistence collaborators. Evidence SHALL retain exact owner identity, positive retrieval controls and restart eligibility. Missing setup, unsupported durability or an unobserved required control SHALL remain BLOCKED rather than becoming a PASS.

#### Scenario: History retrieval across restart

- **WHEN** two owned runs are recorded and the eligible main and store restart under the approved fixture
- **THEN** the discriminator compares GET and list for the exact owner, retains actual results and confirms that foreign-owner or anonymous requests do not expose the records

#### Scenario: Delete one knowledge document

- **WHEN** documents A and B both retrieve before deleting A under the isolated knowledge fixture
- **THEN** the discriminator requires A's captured IDs to disappear while B and the separate knowledge-base control persist, without treating zero hits before and after as success

### Requirement: CAS evidence distinguishes a losing transition from unrelated conflict

The exact-row CAS discriminator SHALL record both forced orderings, winner revision and readback, and SHALL distinguish the losing authorized transition from an unrelated transaction conflict. Unrelated conflict errors SHALL NOT be swallowed as a successful losing transition.

#### Scenario: Both orderings contend for one row

- **WHEN** the approved fixture forces competing authorized transitions in each ordering against the same owned row
- **THEN** evidence identifies the winning revision and readback, permits exactly one terminal effect and records the loser's actual outcome

#### Scenario: Contention is absent or unrelated

- **WHEN** the intended schedule cannot be observed or a separate transaction conflict occurs
- **THEN** the missing schedule remains BLOCKED and the unrelated conflict remains an error rather than causal CAS proof

### Requirement: Historical B3 disposition preserves missing evidence honestly

The finite BTHREE-01 disposition SHALL identify the located or missing original archive and its source pointers. An absent original archive SHALL produce runtime NOT_EXERCISED, SHALL NOT become product PASS and SHALL NOT transfer an older sidecar grade to current-main behavior.

#### Scenario: Original archive is absent

- **WHEN** the finite historical search does not locate the specific original B3 archive
- **THEN** the receipt records the attributed absence and NOT_EXERCISED without inventing runtime evidence or expanding into the whole historical certification campaign

#### Scenario: Archive is located but not reviewed for the candidate

- **WHEN** a historical archive is available without the required candidate-bound review
- **THEN** the driver retains a BLOCKED disposition rather than treating the archive's presence as current candidate acceptance
