## Purpose

Bounds the memory a long-lived UAR process spends on finished runs and idle sessions, by defining when a terminal run and its event and A2UI histories are removed, when an idle session is removed, and what a caller sees afterwards.

## ADDED Requirements

### Requirement: Terminal runs are evicted after a retention period
UAR SHALL remove a run's record, dialogue copy, event history (including any eviction base) and A2UI replay history when all of these hold: the run reached a terminal state (finished, error or cancelled); no stream subscriber is attached; no descendant run that shares its bindings is still running; and the configured retention period has passed since the terminal state. The default retention period SHALL be longer than the runs-stream resync grace period. A run that is not terminal SHALL never be evicted.

#### Scenario: Finished run after retention
- **WHEN** a run finished, has no subscriber, and the retention period has passed
- **THEN** its stream, resume, cancel, approval, checkpoint listing and A2UI routes answer 404

#### Scenario: Subscriber still attached
- **WHEN** a terminal run's retention period passes while a subscriber is attached
- **THEN** the run stays retained until that subscriber detaches and the period has passed again from the detachment

#### Scenario: Running run
- **WHEN** a run has been running longer than the retention period
- **THEN** it is not evicted

### Requirement: Retained terminal runs are capped
UAR SHALL retain at most the configured number of terminal runs. When the cap is exceeded, UAR SHALL evict terminal runs with no attached subscriber in order of their terminal time, oldest first, before their retention period ends, and SHALL expose the number of retained runs as a metric.

#### Scenario: Cap exceeded
- **WHEN** the cap is 3 and five runs finish with no subscriber
- **THEN** the two runs that finished first answer 404 and the other three remain

### Requirement: An evicted run is indistinguishable from an unknown run
Routes addressed to an evicted run id SHALL respond as for an unknown run id and SHALL NOT start a new run. A session whose most recent run was evicted SHALL keep its session history for as long as the session itself is retained.

#### Scenario: Continue after eviction
- **WHEN** a host posts an A2UI action for a surface of an evicted run
- **THEN** the response is 404 and no continuation run starts

#### Scenario: Next turn after eviction
- **WHEN** the most recent run of session S was evicted, session S is still retained, and the host starts a new run for S
- **THEN** the new run sees the session's prior turns

### Requirement: Idle sessions are evicted
UAR SHALL remove a session from its in-memory session store when the session has no non-terminal run and its last activity is older than the configured idle timeout. When more sessions than the configured maximum remain, UAR SHALL remove further sessions that have no non-terminal run, least recently active first. A session with a non-terminal run SHALL NOT be removed, and starting a run on a session SHALL count as activity. Both limits SHALL be configurable, and a value of zero SHALL disable that limit. In sidecar mode, unless the operator configured them, the idle timeout SHALL default to 1800 seconds and the maximum to 1000 sessions; in standalone mode both SHALL default to disabled. Sessions SHALL be evicted per owner and session id, so eviction under one principal SHALL NOT affect another principal's session with the same id.

#### Scenario: Idle session in sidecar mode
- **WHEN** a sidecar session has no running run and no activity for longer than the idle timeout
- **THEN** it is removed from the session store and the active-session count drops by one

#### Scenario: Session waiting on an approval
- **WHEN** a session's run has been paused on a tool approval for longer than the idle timeout
- **THEN** the session is not removed

#### Scenario: Cap exceeded
- **WHEN** the maximum is 2 and three sessions without running runs exist, used in the order A, B, C
- **THEN** session A is removed and sessions B and C remain

#### Scenario: Standalone default
- **WHEN** standalone UAR runs without session retention settings
- **THEN** no session is removed for idleness or by a cap

### Requirement: An evicted session is re-seeded from host history
After a session is evicted, the next run for the same owner and session id SHALL start from an empty session. When that run carries host-supplied history, UAR SHALL seed the empty session from it and report the history as seeded; when it carries none, the run SHALL start with no prior turns and report that no history was used, as after a restart. UAR SHALL NOT restore an evicted session from any other source.

#### Scenario: Next run carries history
- **WHEN** session S was evicted and the host starts a new run for S with S's prior turns as history
- **THEN** the run reports the history as seeded and its model request contains those prior turns

#### Scenario: Next run carries no history
- **WHEN** session S was evicted and the host starts a new run for S without history
- **THEN** the run's model request contains no turn from before the eviction and the run reports that no history was used
