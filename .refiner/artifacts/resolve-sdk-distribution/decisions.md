# Refinement decisions — `resolve-sdk-distribution`

### Iteration 1 decision

- **Decision:** continue
- **Iteration:** 1 of 5
- **Blocking violations remaining:** 3
- **Rationale:** workflow-policy preservation was false, lockfile churn was not
  justified or verified, and artifact state/evidence were incomplete.
- **Next focus:** retire the routine workflow, prove the exact final lock, and
  persist a complete candidate before re-review.

### Iteration 2 decision

- **Decision:** continue
- **Iteration:** 2 of 5
- **Blocking violations remaining:** 1
- **Rationale:** the judge passed, but the critic proved runtime-first was not a
  complete publication order because the runtime has four path-only normal
  dependencies.
- **Next focus:** record the full prerequisite/remediation chain and stop
  claiming the Rust SDK is registry-publishable today.

### Iteration 3 decision

- **Decision:** terminate
- **Iteration:** 3 of 5
- **Blocking violations remaining:** 0
- **Rationale:** the remaining semantic finding is closed; both independent
  reviewers pass the release-ordered, no-publishable-now candidate.
