# Harness protected context task 1.2 baseline

The frozen checkout is exactly `226d4a0af89811975662cf3f203c699f70e8cebc`. Its only changes are the two test-adapter files, whose combined SHA-256 is `5aa484538a593b143b49e1f27aeff2be09938132bcba8d2cc83049bd25703ed3`. The implementation checkout has the same revision and adapter hash, with no diff under `src`, `crates`, `Cargo.toml`, `Cargo.lock`, `versions.toml` or `frontend` attributable to this task.

The active history suite passed 15 tests with two diagnostics ignored. Running those two diagnostics explicitly passed and reproduced both observed failures: SlidingWindow drops the protected empty-text assistant/tool-result group, and pre-history display truncation removes protected middle bytes. The final-request capture control passed in the active suite. The required context-strategy suite passed all four tests.

The successful verification commands used `RUSTC_WRAPPER=` after one observed `sccache` client stall. The interrupted wrapper-backed attempt is recorded as infrastructure history and is not counted as passing evidence. `git diff --check` and `cargo fmt --all -- --check` also exited zero.
