# Tasks — project-instructions-world-state

scope: src/uar/runtime/world_state/**, src/uar/runtime/project_instructions.rs, src/uar/runtime/turn/contributors.rs (world-state contributor), src/config.rs, tests/project_instructions.rs, tests/world_state_diff.rs

## 1. Failing tests first

- [x] 1.1 `tests/project_instructions.rs`: a tree with `AGENTS.md` at root and in `crates/a/` yields root-then-subdir concatenation with the separator when cwd is `crates/a`; parent of root is never read
- [x] 1.2 `AGENTS.override.md` beside `AGENTS.md` wins; an untrusted workspace yields no instructions
- [x] 1.3 A subtree `AGENTS.md` is loaded on the first file read inside that subtree, not at turn start
- [x] 1.4 `tests/world_state_diff.rs`: with a substituted clock, the first turn renders every section in full; a cwd change on turn 2 renders only the environment section's diff; an unchanged turn inside the same one-minute bucket renders nothing; advancing the clock into the next bucket renders only the time section
- [x] 1.5 After a history rewrite, the next turn renders every section in full
- [x] 1.6 A project instruction that imitates a policy line renders inside `Host` markers and the `Policy` fragment hash is unchanged

## 2. Discovery

- [x] 2.1 Add `project_instructions.rs`: root-marker walk, file-name list, override preference, trust check, subtree-on-read hook
- [x] 2.2 Config: `project_instructions.file_names`, `root_markers`, `trusted_workspaces`; defaults documented

## 3. World state

- [x] 3.1 Add `world_state/sections.rs` with stable ids for environment, time, permissions, project instructions and their replacement and removal texts; the time section takes a `Clock` trait and a configurable granularity (default 60 s)
- [x] 3.2 Add `world_state/merge_patch.rs`: RFC 7386 generator and applier; nulls mean deletion; arrays replaced whole
- [x] 3.3 Contributor: full render when no baseline or after rewrite; diff otherwise; baseline invalidated on compaction

## 4. Verification

- [x] 4.1 Tier 1: the two new test targets
- [x] 4.2 Tier 2: fmt check and full test run
- [x] 4.3 `openspec validate project-instructions-world-state --strict`
