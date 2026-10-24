# Tasks

Implementation boxes record authored production artifacts only; they do not claim tests passed. Verification is prohibited until the explicit operator-approved whole-phase boundary.

## 1. Workflow and contract

- [x] 1.1 Register approved issue scope, source pin, ownership and CLOSED verification boundary in canonical KBD and OpenSpec; evidence is the registered task ledger and contract artifacts.

## 2. Reproducible source upgrade

- [x] 2.1 Export pinned facade and required workspace/path crates with licenses and provenance; evidence is the source export and documented manifest adaptations.
- [x] 2.2 Integrate minimal root dependency features and lockfile changes without toolchain churn; reproducible resolution remains unverified until the approved gate.

## 3. Bounded production admission

- [x] 3.1 Implement slot admission, bounded descriptor queue, FIFO, deduplication and durable overflow recovery; evidence is the production adapter and authored admission scenarios.
- [x] 3.2 Migrate executor/errors/metadata and retain grants through actual joins, failure settlement and ordered shutdown; evidence is the production ownership path and authored lifecycle scenarios.
- [x] 3.3 Author finite production-path admission/durability/lifetime acceptance and v1.3.0 integration documentation; tests remain NOT RUN.

## 4. Version and handoff

- [x] 4.1 Bump root UAR to 1.3.0, its lockfile identity and changelog after functional implementation; evidence is the final source-bound handoff inventory.
- [x] 4.2 Collect writer handoffs and reconcile canonical whole-phase implementation state, remaining work and exact proposed gate commands; evidence is a new inventory that does not imply approval.

## 5. Operator-gated acceptance

- [ ] 5.1 After explicit command approval, run the local finite integration and supported-profile/customer-build gates serially and record exact source/profile/artifact outcomes.
- [ ] 5.2 After permission opens the review boundary, obtain independent artifact-only Rust/code review and resolve findings without claiming earlier acceptance for changed source.

## Workflow follow-up

- Archive/sync the OpenSpec change only after acceptance and independent review are complete.
- Push, PR publication, tags and release publication require their own explicit authorization.
