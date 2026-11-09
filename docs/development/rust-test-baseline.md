# Local Rust library test baseline

Issue #462 exposed evidence produced after a temporary test-source repair. A
successful compile of that patched tree did not establish that the committed
candidate's library tests compiled. This local safeguard keeps that distinction
visible and never patches source, resets dependencies, or changes release tooling.

Run from the checkout:

```bash
bash scripts/verify-rust-test-baseline.sh
```

The script serially runs these compile-only commands and stops at the first error
or warning diagnostic:

```bash
cargo test --locked --no-default-features --features minimal --lib --no-run
cargo test --locked --no-default-features --features server-full --lib --no-run
cargo test --locked --no-default-features --features server-full,postgres-backend --lib --no-run
```

Each command uses `CARGO_TERM_COLOR=never` so its visible diagnostics can be
checked for `warning:` and `warning[code]:` lines. Full output remains visible;
warnings fail the safeguard without suppressing lints or changing `RUSTFLAGS`.

Lefthook selects the command for staged Rust files, Cargo manifests/locks, the
root Rust toolchain/config files, and the safeguard itself. Separate root and
nested globs follow [Lefthook's documented glob behavior](https://lefthook.dev/configuration/glob/).
The command compiles the **working tree**, including unstaged edits. Its success
does not certify a partially staged commit. It does not stash or stage changes.
This safeguard is local; it adds no GitHub Actions job.

For acceptance evidence, first commit the candidate and ensure the checkout and
every recursive submodule are clean, initialized, and at their committed pins:

```bash
bash scripts/verify-rust-test-baseline.sh --acceptance
```

Acceptance mode writes commit/tree IDs, recursive submodule pins, Cargo/rustc
versions, target directory, test profile, feature sets, and exact commands to
stdout. Save that output outside the source tree or in an ignored output
directory. It re-reads the source and toolchain identities after compilation and
refuses a pass if they changed or the source became dirty. Git, toolchain, and
Cargo errors fail the command. A dirty checkout or mismatched/uninitialized
submodule refuses acceptance before compilation. Do not temporarily patch source
for this run. The pins recorded here are Git's committed pins; this script does
not reconcile policy metadata such as `versions.toml` or repin dependencies.

The test profile inherits the repository's dev profile. Cargo still consumes
local configuration and environment; this is not a hermetic build receipt or
proof that source was never transiently edited between the two identity reads.
Keep the candidate frozen during the run and retain the full diagnostics,
including warnings, for phase acceptance. Compilation alone proves no test
executed and no runtime behavior, provider behavior, or release qualification.

The script honors `CARGO_TARGET_DIR` (default `target`) and holds a directory lock
there across the three commands. Other invocations of this safeguard fail while
that lock exists. Cargo's own locks and the operator's single-writer scheduling
still apply to other build commands. After an interrupted process, remove
`.uar-rust-test-baseline.lock` only after confirming its owner has stopped.

Repository verification locks still apply: do not run this script while the
phase's gate is closed. It is an executable verification command, including when
invoked by pre-commit.
