# Tasks

Complete production work in C06.1-C06.3 order. A task's named observable behavior is checked by static inspection during implementation and by the **single completed-phase integration gate in 3.5**; it is not permission to run per-task tests or builds. The UAR product receipt links these groups to initiative C06.1-C06.3.

## 1. C06.1 — Durable identity and activation profiles

- [ ] 1.1 Pin the C05 UAR revision, accepted C01-C04/P1 contracts, source file claims and SurrealDB 3.3.0 store capability in the C06 implementation receipt; verify the recorded source and actual provider match before application code changes.
- [ ] 1.2 Add a focused, versioned owner/workspace-scoped logical-instance record and Surreal-backed conditional store, separate from collaboration catalog and service-instance identity; verify an implementation trace from authenticated owner/workspace to stored instance revision and exact definition/binding.
- [ ] 1.3 Add explicit request, on-demand and opt-in resident profiles above the existing actor/thread host, creating a fresh root per turn; verify a trace from one logical instance through two distinct root IDs without a second model loop or reusable root.
- [ ] 1.4 Expose the durable profile only for a provider with proven restart storage; memory remains non-durable and Postgres is either implemented against the same contract or refused; verify capability discovery and admission represent the actual selected provider.

## 2. C06.2 — Bounded inbox, serialization and fencing

- [ ] 2.1 Persist finite inbox/retention limits, command IDs, idempotent receipts and turn/root associations with owner/workspace-scoped conditional writes; verify duplicate-identical, duplicate-conflicting and full-inbox paths in the completed gate.
- [ ] 2.2 Serialize mutating turns per instance while preserving independent authorized status/cancel control; verify a blocked tool cannot admit another mutating turn yet does not block cancellation or inspection.
- [ ] 2.3 Add monotonic single-host activation epochs to state and protected-effect commits, reconcile exact root/effect receipts on restart, and retain uncertain outcomes; verify a stale activation cannot commit and an uncertain effect is never replayed as safe.

## 3. C06.3 — Lifecycle, administration and completed gate

- [ ] 3.1 Implement revisioned activate/passivate/drain/disable/restart with repeat-safe hooks, finite retry budget and explicit queued/active-work dispositions; verify failure exhaustion, disabling and recovery are inspectable after process restart.
- [ ] 3.2 Add authenticated instance administration and additive committed event projection with replay-gap/snapshot behavior; verify owner/workspace isolation, redaction and stable instance/command/attempt/root attribution.
- [ ] 3.3 Preserve ordinary run, actor and A2A behavior; advertise a distinct durable-instance capability only after backing semantics are operational, without claiming TeamInstance activation or cross-host takeover; verify old clients still execute and capability discovery is truthful.
- [ ] 3.4 Record storage migration, rollback and an operator recovery path for unsupported or uncertain records; verify versioned records survive a cold process restart and downgrade does not reinterpret them as ordinary runs.
- [ ] 3.5 Run one local real-host, real-SurrealDB 3.3.0 integration gate after all production tasks above are complete, covering every scenario in `specs/durable-agent-instances/spec.md` plus exact duplicate/conflict, two workspaces, distinct root IDs, crash/passivate/reactivate, stale epoch, blocked-tool cancel/status, capacity/retry exhaustion, event gap, ordinary-client compatibility and uncertain-effect refusal; verify one receipt records exact source, provider, policy/binding revisions and observed outcomes, then reconcile initiative C06.1-C06.3 without claiming team or federation conformance.
