## Context

See `proposal.md` for motivation. UAR consumes KBD from the `crates/prometheus-skill-system` gitlink, while the installed `prometheus` CLI is built from that upstream source. The repository already separates KBD project registration from signed run initialization.

## Goals / Non-Goals

**Goals:**

- Pin the exact reviewed upstream implementation that repairs issue #265.
- Preserve the UAR KBD audit and legacy phase state across first initialization.
- Certify the installed CLI through the same isolated process tests as the source build.

**Non-Goals:**

- Changing UAR backend, frontend, provider, persistence, payload, or realtime behavior.
- Adding Unix-socket transport or restarting `sovereign-sync`.
- Initializing canonical history during registration or read-only status.
- Weakening migration guards that protect projection-only completion evidence.

## Decisions

### Pin the reviewed upstream commit

Advance only the `crates/prometheus-skill-system` gitlink to upstream commit `602750ec61bc4674b51231fb36f3bfee3af42b7e`. This keeps KBD implementation ownership upstream and makes the UAR dependency exact. Copying the runtime patch into UAR would duplicate ownership and diverge installed tooling from the pinned source.

### Keep deployment scoped to the CLI

Install and ad-hoc sign the rebuilt `prometheus` CLI. The daemon does not contain the changed code, so retaining the running `sovereign-sync` process is safer and more accurate than restarting an unchanged service.

### Preserve issue audit until merge

Link the upstream and UAR review PRs from issue #265 and leave it open until the repository pin merges. Deleting the issue would remove useful failure history, while closing it before merge would represent a review branch as delivered repository state.

## Risks / Trade-offs

- [Upstream review commit is not yet on upstream `main`] → Pin its exact pushed commit and keep the UAR issue open until review dependencies merge.
- [Installed binary needs rollback] → Preserve the prior signed CLI under `/Users/gqadonis/.local/bin/backups/` before atomic replacement.
- [Legacy projections contain completion without equivalent rows] → Preserve the projection-ahead refusal and report the existing broad-suite baseline rather than discard evidence.

## Migration Plan

Merge upstream PR #68, then merge the UAR pin PR. Existing initialized runtimes remain unchanged; registered empty runtimes initialize only at their next typed mutation. Roll back by restoring the prior UAR gitlink and the preserved CLI backup; signed initialization events already committed remain immutable history.
