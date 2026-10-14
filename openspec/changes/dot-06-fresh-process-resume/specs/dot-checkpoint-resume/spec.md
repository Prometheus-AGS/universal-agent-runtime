# Spec Delta

## Purpose

Define checkpoints restore authorized execution in a fresh process for reliable always-on assistant operation and honest support claims.

## ADDED Requirements

### Requirement: Checkpoints restore authorized execution in a fresh process

Checkpoint resume SHALL reconstruct authorized execution from durable protected metadata under a single incarnation-bound continuation claim. Unsupported persistence SHALL fail explicitly. Uncertain predecessors SHALL block takeover until trusted exclusion and reconciliation.

#### Scenario: Fresh manager

- **WHEN** the original process is gone and an authorized owner resumes a durable checkpoint
- **THEN** the runtime reconstructs from protected persisted metadata after current authority and host-resource checks

#### Scenario: Unavailable capability

- **WHEN** the selected backend cannot persist or load resumable checkpoints
- **THEN** the request fails explicitly and never claims a durable save or a successful empty resume

#### Scenario: Concurrent checkpoint resume

- **WHEN** two authorized runtimes resume the same checkpoint revision concurrently
- **THEN** one durable incarnation-bound claim admits at most one continuation, and uncertain predecessors remain blocked until trusted exclusion and reconciliation


