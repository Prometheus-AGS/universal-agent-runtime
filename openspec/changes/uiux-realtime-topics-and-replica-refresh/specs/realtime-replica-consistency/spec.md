## ADDED Requirements

### Requirement: Every public topic has a change source on every supported store
Each public realtime topic SHALL be fed by a change source on embedded Surreal, remote Surreal and Postgres. This SHALL include `conversation_policies`, fed by the legacy and principal conversation policy tables, and `team_catalog`, fed by the collaboration catalog table.

#### Scenario: Conversation policy change publishes on a remote store
- **WHEN** a conversation policy is saved through one replica and a client holds `/api/live` on another replica of the same remote Surreal or Postgres store
- **THEN** the client receives a `reset` frame for `conversation_policies`

#### Scenario: Conversation policy delete publishes
- **WHEN** a conversation policy is deleted through one replica
- **THEN** a client on another replica receives a `reset` frame for `conversation_policies` and that replica no longer returns the deleted policy

#### Scenario: Team catalog change publishes
- **WHEN** a deployment binding is installed through one replica
- **THEN** a client on another replica receives a `reset` frame for `team_catalog`

#### Scenario: Embedded store publishes the same topics
- **WHEN** a conversation policy is saved on an embedded Surreal deployment
- **THEN** a client on `/api/uar/sync/stream` receives a `reset` frame for `conversation_policies`

### Requirement: Provider changes invalidate providers and models on every store
A change to a `provider.*` setting SHALL invalidate `providers` and `models`, and a change to `llm.default_provider` SHALL invalidate `settings` and `providers`, identically on Surreal and Postgres. A change to any other setting SHALL invalidate `settings` only.

#### Scenario: Provider edit on Surreal invalidates providers
- **WHEN** a provider is edited on a Surreal store
- **THEN** clients receive `reset` frames for `providers` and `models` and none for `settings`

#### Scenario: Default provider change invalidates providers
- **WHEN** the default provider is changed on either store
- **THEN** clients receive `reset` frames for `settings` and `providers`

#### Scenario: Unrelated setting does not invalidate providers
- **WHEN** a setting outside `provider.*`, `model.*` and `llm.default_provider` changes
- **THEN** clients receive a `reset` frame for `settings` and none for `providers` or `models`

### Requirement: Invalidation frames carry no row data
A realtime frame SHALL carry only its version, topic, snapshot requirement, reason, generation and readiness. It SHALL NOT carry a record id, an owner, a key or a value.

#### Scenario: Frame body is minimal
- **WHEN** any public topic emits a frame
- **THEN** the body contains none of the changed record's identifier, owner, key or value

### Requirement: Each topic declares whether a snapshot is required
Each topic SHALL declare whether clients must fetch a snapshot. Owner-private topics without a list route SHALL declare that no snapshot is required, and their frames SHALL say so. Clients SHALL then refetch the records they hold through the existing owner-scoped read routes.

#### Scenario: Owner-private topic is snapshotless
- **WHEN** a `conversation_policies` or `team_catalog` frame is emitted
- **THEN** it carries `requires_snapshot: false`, and a call to that topic's snapshot route answers `complete: false` with an owner-scoped-refetch reason

#### Scenario: Open conversation refetches its policy
- **WHEN** a conversation's policy is changed from another place while the conversation is open
- **THEN** the open conversation shows the new policy without a page reload

### Requirement: Each replica refreshes its in-memory provider and settings state from the change feed
Each replica SHALL subscribe server-side to the change feed and, on a `providers`, `settings` or `models` change, a feed reset, a lag or startup, SHALL reload its settings cache from the store and replace its provider registry and default from the stored rows. A provider or default-provider change made through any replica SHALL be the state every replica serves for its next snapshot and next turn.

#### Scenario: Default provider change takes effect on another replica
- **WHEN** the default provider is changed through replica A of two replicas sharing one remote store
- **THEN** replica B reports the new default and its next model-less turn uses that provider's default model

#### Scenario: Provider deletion reaches another replica
- **WHEN** a provider is deleted through replica A
- **THEN** replica B no longer lists it and no longer routes to it

#### Scenario: Settings change reaches another replica
- **WHEN** a next-turn setting is changed through replica A
- **THEN** replica B returns the new value from its settings reads

### Requirement: Replica refresh never applies governance or restart-class keys
Refresh SHALL NOT change governance posture, which is published only by its runtime authority. It SHALL NOT report a `restart` or `inert` key as in effect, using the settings change's classification.

#### Scenario: Restart-class key stays restart-required
- **WHEN** `memory.max_context_tokens`, a restart-class setting, is changed through replica A
- **THEN** replica B reports the stored value with status `restart_required` and has not applied it to its running configuration

### Requirement: The registry default follows the stored default provider row on every replica
Each replica SHALL set its registry default from the stored `llm.default_provider` row when that provider exists. That row SHALL be written only by the live default route and by deletion of the default provider; the settings API SHALL reject writes to it.

#### Scenario: Default row applied on every replica
- **WHEN** the live default route changes the default provider on replica A
- **THEN** replica B's registry default equals the stored row within the convergence deadline

#### Scenario: Settings API cannot reach the row
- **WHEN** a client writes `llm.default_provider` through the settings API
- **THEN** the write is rejected as defined by the settings change and no replica's default changes

### Requirement: A failed persist publishes nothing and changes no replica
A default-provider change SHALL persist before the live registry swap and before any publication. When the persist fails the request SHALL be rejected, no replica's default SHALL change, and no topic frame SHALL be published.

#### Scenario: Persist fault leaves every replica unchanged
- **WHEN** the default route is called on replica A while A's store connection fails
- **THEN** the route answers non-2xx, the default and stored row are unchanged on A and B, and B's client receives no `providers` or `settings` frame

#### Scenario: A swap before the persist is a defect
- **WHEN** the live default is swapped before the durable write and the write then fails
- **THEN** the acceptance gate reports a failure because replica A's default differs from the stored row

#### Scenario: Route works after the fault clears
- **WHEN** the store connection is restored and the same request is repeated
- **THEN** the default changes on the replica that received the request and clients receive a settings frame

### Requirement: Agent-kernel paths cannot mutate settings or the default provider
An agent kernel SHALL NOT change settings or the default provider. The tool catalog exposed to agents SHALL contain no settings or provider mutation tool, and the settings and provider mutation routes SHALL require host admin authority that no agent run holds, independently of any network-address rule.

#### Scenario: Kernel tool call to the default route is refused
- **WHEN** an agent run's tool call POSTs to the replica's own default-provider route
- **THEN** the tool call returns an error, the default and stored row are unchanged on every replica, and no frame is published

#### Scenario: Refusal does not depend on the address rule
- **WHEN** an agent run's request reaches the default-provider route without the host admin key on a deployment that requires admin authority
- **THEN** the route answers 401 and no replica's default changes

#### Scenario: A deployment that does not require admin authority is detectable
- **WHEN** the same request reaches the route on a deployment that does not require admin authority
- **THEN** the default changes, which the acceptance gate reports as a failure of the guard

#### Scenario: MCP tool surface has no mutation tool
- **WHEN** the tool list of the UAR runtime MCP endpoint is read
- **THEN** it contains no tool whose name refers to settings, providers, default model or configuration

### Requirement: A snapshot is served only after the replica applied the change it was told about
A replica SHALL stamp a per-topic change epoch before broadcasting a change, and the snapshot routes for `providers`, `models` and `settings` SHALL wait until the replica has applied that epoch. A client that received a frame SHALL NOT then read the pre-change state from that replica.

#### Scenario: Frame then snapshot is current
- **WHEN** a client on replica B receives a `providers` frame for an edit made through replica A and immediately requests the providers snapshot from B
- **THEN** the snapshot reflects the edit

#### Scenario: Applying is slow
- **WHEN** the replica has not applied the epoch before the admission deadline
- **THEN** the snapshot answers `complete: false` with a pending reason and not a complete stale record set

### Requirement: A replica resynchronizes after a feed gap and says so while stale
A replica SHALL perform a full refresh when its feed resets with readiness restored. While its feed is unavailable or its refresh is failing, the snapshots for `providers`, `models` and `settings` SHALL answer `complete: false` with a reason, and SHALL answer a complete snapshot again after a successful refresh.

#### Scenario: Missed changes are recovered
- **WHEN** replica B's feed is interrupted, changes are made through replica A, and the feed is restored
- **THEN** replica B serves the new provider and settings values

#### Scenario: Stale replica does not claim completeness
- **WHEN** replica B's feed is unavailable
- **THEN** its providers snapshot is `complete: false` with a reason

### Requirement: The Postgres backend is built and verified
The final integration gate SHALL build with the `postgres-backend` feature and SHALL exercise every scenario of this capability against a Postgres store as well as a Surreal store.

#### Scenario: Postgres replicas share a store
- **WHEN** two replicas built with `postgres-backend` run against one Postgres database
- **THEN** a provider change through one is served by the other and both report their notification listener ready

### Requirement: Cross-replica behavior is verified with at least two replicas on one remote store
Acceptance of replica consistency SHALL run at least two production replicas against one remote store and SHALL observe each change through a different replica from the one that made it. A single-process run SHALL NOT satisfy it.

#### Scenario: Single process is insufficient
- **WHEN** a gate run uses one replica
- **THEN** the replica-consistency scenarios are reported `NOT_EXERCISED`
