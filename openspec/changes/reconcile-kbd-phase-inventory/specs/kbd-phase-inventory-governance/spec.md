## Purpose

Defines how a KBD phase estate is reconciled so canonical status remains truthful while historical evidence and unique Git work are protected.

## ADDED Requirements

### Requirement: Evidence-backed phase disposition
The reconciliation SHALL assign every registered top-level phase a terminal or retained disposition based on its canonical state, phase artifacts, and named successor outcomes. A partially delivered phase SHALL NOT be represented as fully delivered unless the disposition records how its remaining work was completed or explicitly abandoned.

#### Scenario: Completed phase has direct evidence
- **WHEN** a phase reflection records that its required outcomes were delivered
- **THEN** the canonical phase status is reconciled to complete and the evidence source is recorded

#### Scenario: Successor phase closes remaining outcomes
- **WHEN** a legacy phase has incomplete outcomes that a named successor phase completed
- **THEN** the legacy phase may be terminalized only with the successor relationship and remaining-outcome disposition recorded

#### Scenario: Planned work is no longer relevant
- **WHEN** a phase did not execute or its intended work was superseded, cancelled, or deliberately abandoned
- **THEN** the canonical phase status is reconciled to cancelled rather than complete

### Requirement: Legal canonical transitions
The reconciliation SHALL update phase state through the KBD canonical transition interface and SHALL leave no stale active cursor after all retained work is terminal.

#### Scenario: Pending phase is completed
- **WHEN** evidence supports completing a phase whose canonical state is pending
- **THEN** the phase passes through every state required by the KBD transition rules before reaching complete

#### Scenario: No active phases remain
- **WHEN** every registered phase has a terminal disposition
- **THEN** the run reports no active phase and offers the new-phase entry point

### Requirement: Git cleanup protects unique work
The reconciliation SHALL resolve phase-associated worktrees and local branches by exact path and ref, and SHALL remove them only after checking dirty files, ignored `.prometheus` history, unique commits, and unmerged changes.

#### Scenario: Obsolete phase worktree is safe to remove
- **WHEN** a phase-associated worktree and branch are clean, contain no unique history, and are no longer required by a retained phase
- **THEN** the exact worktree and branch are removed through the repository worktree workflow

#### Scenario: Worktree contains unique work
- **WHEN** a worktree or branch contains dirty, ignored, unique, or unmerged content
- **THEN** it is retained or its work is preserved before cleanup, and the reason is reported

#### Scenario: Unrelated worktree is present
- **WHEN** a local worktree cannot be associated with a reconciled KBD phase
- **THEN** the reconciliation does not delete it as phase cleanup

### Requirement: Reconciliation remains auditable
The reconciliation SHALL preserve historical phase artifacts and SHALL produce a verification record containing every phase disposition, Git cleanup decision, canonical status result, and checks actually run.

#### Scenario: Operator reviews the cleanup
- **WHEN** reconciliation completes
- **THEN** the operator can trace each terminal status and retained or removed Git object to recorded evidence
