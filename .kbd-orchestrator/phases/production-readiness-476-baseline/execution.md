# Execution ownership and boundary
Backend: Codex team + OpenSpec. Lead owns waypoint, baseline source, hook/script, phase records and one serialized Cargo target. Research teammates own only .agent-team/uar-core/production-readiness-{contract,baseline}-research.md. Other writers share this checkout; preserve their files.

Classification: source reads/Git identity/submodule materialization are inspection/setup; phase/spec/handoff edits are authorized documentation; baseline source/script/hook edits are implementation. Tests/builds/lint/review-driven verification are deferred until all five implementation items above are complete, except final verification item itself. No teammate runs checks.

Original parent work remains historical unfinished. The new operator-directed phase is scoped to #462, not a claim that the prior 170-task program completed. Exact source identity will be recorded at boundary. No acceptance has run.

## Acceptance resume
Operator explicitly requested checking shared-build availability and resuming #462, then subsequent bounded phases without routine approval. Original-checkout Cargo PID40283 remains active with rustc child51195; preserve it and wait. Reused the original root node_modules via an ignored worktree-local symlink for existing Lefthook/commitlint dependencies; no install or source/dependency mutation. Current phase production implementation is complete per inventory-002; only acceptance/receipts remain. Next capability slice is source-planning only while the shared compiler writer runs.

Dependency setup correction: Git directory-only node_modules/ ignore does not ignore a root symlink. Replaced only our newly-created root symlink with an actual ignored node_modules directory whose entries link to existing installed dependencies. No original dependency bytes changed.

A serial acceptance entrypoint is queued: source-inspection polling of process executable names waits until no Cargo/rustc process remains; only then ordinary git commit invokes existing local policy and Rust baseline hooks. No external process is killed and no shared config/cache is changed. Source is frozen while that entrypoint runs. Its ordinary hooks compile working-tree source; committed-candidate acceptance follows separately on success. Planned output: target/production-readiness-476/commit-attempt-001.log. The required dependency audit already ran against unchanged dependencies and its findings are retained, not hidden.
