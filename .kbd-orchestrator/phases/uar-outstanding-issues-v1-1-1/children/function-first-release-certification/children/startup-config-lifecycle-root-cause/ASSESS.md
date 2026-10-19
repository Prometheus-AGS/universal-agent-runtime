# ASSESS — startup-config-lifecycle-root-cause

Date: 2026-10-19 14:16 UTC. Candidate: v4 sha256 `96bf4868…5154` (server-full
release). Source: dirty checkout (#419 + checkpoint repair + lifecycle edits),
uncommitted by design.

## Current state inventory

- Parent `function-first-release-certification`: configuration gate NOT READY
  (unwaived) — sole open leg: query-time space-mismatch guard UNVERIFIED (no
  second honored embedding space buildable in this deployment).
- Already-proven boundary evidence (receipts:
  `receipts/config-lifecycle-map-2026-10-19.md` §9):
  - P1 provider credential lifecycle on the no-key deployment class — 5/5 legs.
  - P2 supported mutation/reset subchecks — honored-value live activation with
    next-request proof; unhonored 422 with unchanged row; boot-overlay refusal
    of unhonored drift; ingest-time space binding (KV readback).
  - P3 rejection/reporting — `rag.embedding_provider` 422; `rag.chunk_size`
    200 + `restart_required`.
  - P4 memory boot wiring from rows, live `active=true` contrast,
    disable/restart round trip.
- Diagnosis already executed read-only this session (pre-child, preserved):
  swap-writer inventory, backend-construction inventory, manager-instance
  inventory, config-reload authority, companion-listener role, DTO projection
  trace.
- Open question the child must answer: does any REMAINING competing authority
  or captured stale dependency justify one more targeted boundary redesign?

## Constraints

Frozen EXIT-CONTRACT.md (authored before any code decision, now at the
canonical nested path). No new builds beyond v4, no tests, no deploy, no
publication, no provider fallback, no security changes, no manual projection
edits. Budget: ~601/632 consumed at child start; extensions authorized and
recorded if taken.
