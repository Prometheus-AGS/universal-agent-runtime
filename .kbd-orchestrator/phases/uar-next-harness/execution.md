# Execution: uar-next-harness

**Date:** 2026-07-02
**Backend:** OpenSpec (Round 0 is a direct/native hygiene task)
**Agent:** claude-code
**Phase total:** 21 changes across 4 rounds + 1 operator item

## Dispatch Strategy

Execute in dependency-ordered tranches per `plan.md`. Round 0 is a direct task (committing dirty-tree features); Rounds 1–4 map to nested child phases and are opened as OpenSpec changes when the round starts.

| Round | Child phase | Changes | Strategy |
|---|---|---|---|
| 0 | *(parent phase)* | HK0 `commit-live-sse-dualstack` | Direct task: review dirty tree, run tests/fmt, commit focused commits |
| 1 | `foundation-completion` | CH-01 `a2a-grpc-enable`, CH-02 `postgres-credential-store`, CH-03 `provider-health-failover`, CH-04 `prompt-dialect-engine` | Open now; parallel capable after HK0 |
| 2 | `intelligence-completion` | CH-05..CH-11 | Open after CH-04 lands |
| 3 | `spec-v2-distribution` | CH-12..CH-17 | Open after Round 1 foundation |
| 4 | `integration-and-polish` | CH-18..CH-20 | Open after Round 1 foundation |

## Backend confirmation

- **Round 0:** Native/direct — no OpenSpec change; task list inline in this execution file.
- **Round 1+:** OpenSpec — `openspec/changes/<id>/{proposal.md,tasks.md}` will be created via `/opsx:new`.
- **QA gate:** artifact-refiner `/refine-validate` after each change reaches `DONE` (skip only if <3 files, docs-only, or `--skip-qa`).
- **Per-task hooks:** Route task execution through `/kbd-apply` so `task:before`/`task:after` fire; do **not** drive bare `/opsx:apply`.

## Round 0 — HK0 commit-live-sse-dualstack

Scope: review the current dirty tree, verify the dual-stack listener + multiplexed `/api/live` SSE + shared-EventSource frontend adapter compile and pass tests, run `cargo fmt` on affected Rust files, and commit as focused commits.

### Steps
1. Inspect dirty tree (`git status`, `git diff --stat`).
2. Review dual-stack listener changes in `src/server.rs`.
3. Review multiplexed `/api/live` SSE changes in `src/uar/api/live.rs`.
4. Review shared-EventSource frontend adapter + tests.
5. Run `cargo fmt` (include `registry.rs`, `routes.rs`, `ingestion_worker.rs` if needed).
6. Run `cargo build` and `cargo test` (lib tests).
7. Commit as focused commits:
   - feat(server): dual-stack companion listener + multiplexed live SSE
   - feat(frontend): shared EventSource adapter for /api/live
   - chore: fmt registry/routes/ingestion_worker
   - chore: phase state for uar-next-harness
8. Update progress.json: `changes_completed: 1`.

## Round 1 — Open changes now

After HK0 is done, create OpenSpec changes (do not implement yet):

- `/opsx:new a2a-grpc-enable`
- `/opsx:new postgres-credential-store`
- `/opsx:new provider-health-failover`
- `/opsx:new prompt-dialect-engine`

## QA gates per change

All changes require:
- `cargo build` clean
- `cargo clippy --all-targets --all-features` clean (or `#[expect(lint, reason="...")]`)
- `cargo test` green
- `cargo fmt` applied
- Behavioral verification per tasks.md §5 (for OpenSpec changes)

## Progress tracking

- `progress.json` initialized with `changes_total: 21, changes_completed: 0`.
- After HK0: `changes_completed: 1`.
- Round 1 changes opened but not completed; next execution pass targets `foundation-completion`.
