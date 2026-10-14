# Spec Delta

## Purpose

Define accepted uploads own durable recoverable ingestion work for reliable always-on assistant operation and honest support claims.

## ADDED Requirements

### Requirement: Accepted uploads own durable recoverable ingestion work

Upload acceptance SHALL occur only after source content and owner-scoped ingestion work are durably committed together. Accepted work SHALL remain discoverable after process failure.

#### Scenario: Crash after acceptance

- **WHEN** the runtime returns 202 and exits before dispatch
- **THEN** source bytes and one owner-bound ingestion job remain discoverable and recover without duplicate indexed chunks

#### Scenario: Commit failure

- **WHEN** source or job transaction cannot commit
- **THEN** the upload is not accepted and no metadata falsely promises future processing


