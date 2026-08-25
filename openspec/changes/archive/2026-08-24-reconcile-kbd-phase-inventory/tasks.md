## 1. Inventory and Classification

- [x] 1.1 Snapshot the canonical top-level phase list, states, artifacts, and current cursor; verify every registered phase appears exactly once in the disposition ledger
- [x] 1.2 Classify each phase as complete, cancelled, or retained with direct reflection or named-successor evidence; verify no partial phase is called complete without an explicit remaining-outcome disposition

## 2. Canonical Reconciliation

- [x] 2.1 Apply legal KBD transitions for evidence-backed completed phases; verify each reaches canonical `complete`
- [x] 2.2 Apply legal KBD transitions for superseded or abandoned phases; verify each reaches canonical `cancelled`
- [x] 2.3 Resolve any retained relevant phase before cleanup ends; verify no registered phase remains unintentionally active or pending

## 3. Git Estate Cleanup

- [x] 3.1 Map local worktrees and branches to phases and audit tracked dirt, untracked files, ignored `.prometheus` history, unique commits, and merge reachability; verify every Git cleanup decision is recorded
- [x] 3.2 Remove only obsolete phase-associated worktrees and branches through the repository helper; verify `git worktree list` and local refs contain no stale phase-associated entries

## 4. Verification and Handoff

- [x] 4.1 Create the reconciliation verification record with all phase dispositions, successor mappings, and Git decisions; verify its phase count matches the canonical inventory
- [x] 4.2 Run OpenSpec validation, KBD status checks, projection consistency checks, and Git diff checks; record the observed output and any unavailable external synchronization
- [x] 4.3 Commit only scoped reconciliation artifacts and generated canonical projections; verify unrelated dirty files and the protected PR worktree remain unchanged
