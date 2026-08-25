## ADDED Requirements

### Requirement: Post-merge native release cleanup is evidence-gated
After a native macOS fix merges, release closeout SHALL install artifacts built from the synchronized default branch and SHALL verify the installed executable, readiness, canonical settings routes, and durable provider identity before removing related review worktrees, branches, or regenerable build outputs. Worktree cleanup MUST preserve any project memory not already present in the merged repository.

#### Scenario: Merged macOS fix is deployed and cleaned up
- **WHEN** an operator closes out a merged fix by rebuilding and installing the macOS LaunchAgent
- **THEN** evidence identifies the merged source revision and installed executable digest
- **AND** health, readiness, canonical settings reads, and provider identity continuity succeed after restart
- **AND** only merged review worktrees and branches without unique project memory are removed
- **AND** only regenerable build outputs are deleted after their pre-cleanup size is recorded

#### Scenario: Installation verification fails before cleanup
- **WHEN** the installed service fails readiness, canonical settings reads, provider continuity, or executable identity verification
- **THEN** related review worktrees and rollback artifacts remain available
- **AND** the failed installation is reported before any build-output cleanup proceeds
