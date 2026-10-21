# Operator-approved lifecycle command results

Phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Approval: the operator approved ready commands in
`lifecycle-verification-command-proposal.md`, serial execution, and repairs of
observed failures. Commands with unmet prerequisites remain closed. No other
checks are authorized. This supersedes CLOSED only for the exact commands
listed below and their necessary failed-command reruns after source repairs;
each repair must receive a renewed source inventory before its rerun. It does
not grant general verification, policy validation, formatting, lint, typecheck,
extra compiler probes, independent review or broader acceptance.

## Authorized serial sequence

1. The exact pinned MiniLM preparation command in the proposal.
2. The exact `embedding_lifecycle_failover` integration command, after model preparation succeeds.
3. The exact `lifecycle_owned_resources` integration command.
4. The exact `sidecar_launch_security` integration command.
5. The exact native Boss candidate producer at source `32cb3bcfc9691212ec0b0359ce22474968bc6850`.
6. The exact local Boss package command, after its matching candidate exists.

The lead owns all command execution and
the single build writer. Teammates may inspect source and implement explicitly
assigned repairs, but may not execute gates.

## Still closed

- `pnpm build:mac:arm64`: no new matching public sidecar payload exists.
- Full release acceptance runner: its manifest lacks fresh same-image supervisor,
  source/image/fixture binding and independent completed-artifact review.
- Windows commands, which have no approved exact packet.
- All commands absent from the approved ready set.

The Docker deployment command in the proposal is a completed historical
delivery receipt. It is not repeated. No passing command will be broadened.

The uncomfortable limit remains: successful compilation or these individual
targets cannot satisfy every frozen embedding condition or whole release
acceptance. Report target outcomes and cleanup failures separately.

## 1. Model preparation — PASS

Working directory: `/Volumes/T7/Projects/prometheus/universal-agent-runtime`.

```sh
python3 tests/fixtures/local_embedding_lifecycle/prepare_models.py --primary-dir /Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models --incompatible-dir /Volumes/T7/uar-local-embedding-minilm-1110a243fdf4
```

Exit0. Actual output:

```text
Prepared bg-small-en-v1.5.onnx: 90405214 bytes; sha256=6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452
Prepared tokenizer.json: 466247 bytes; git-blob-sha1=cb202bfe2e3c98645018a6d12f182a434c9d3e02
Prepared config.json: 612 bytes; git-blob-sha1=72b987fd805cfa2b58c4c8c952b274a11bfd5a00
Prepared special_tokens_map.json: 112 bytes; git-blob-sha1=e7b0375001f109a6b8873d756ad4f7bbb15fbaa5
Prepared tokenizer_config.json: 350 bytes; git-blob-sha1=c79f2b6a0cea6f4b564fed1938984bace9d30ff0
Prepared pinned local model sentence-transformers/all-MiniLM-L6-v2 at /Volumes/T7/uar-local-embedding-minilm-1110a243fdf4; inference and acceptance remain pending
```

This validates asset preparation and the primary-weight inequality. It does
not establish inference or lifecycle behavior. The passing preparation command
will not be repeated; it deliberately requires a new target directory.

## 2. Embedding integration — first compiled attempt FAILED

```sh
UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR=/Volumes/T7/uar-local-embedding-minilm-1110a243fdf4 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_MODEL=sentence-transformers/all-MiniLM-L6-v2 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIMENSION=384 cargo test --locked --no-default-features --features server-full --test embedding_lifecycle_failover -- --test-threads=1
```

First invocation exited127 before compilation:

```text
/usr/local/bin/bash: line 1: cargo: command not found
```

Filesystem inspection found `/Users/gqadonis/.cargo/bin/cargo` is the installed
rustup launcher. The same approved command is now running with only this
process environment repair before it:

```sh
export PATH=/Users/gqadonis/.cargo/bin:$PATH
```

Arguments, feature profile and test selection are unchanged. No `cargo
--version`, compiler probe or other preflight ran. No source changed. Its
actual compilation/test result follows.

The invocation with Cargo available exited101 before any test ran. The full
actual output is retained in
`lifecycle-verification-output-embedding-2026-10-21-attempt-1.log`. Fatal
diagnostic locations and category:

```text
E0277 LowerHex: tests/embedding_lifecycle_failover.rs:518:38
E0277 LowerHex: tests/embedding_lifecycle_failover.rs:524:21
E0277 LowerHex: tests/embedding_lifecycle_failover.rs:534:21
Compilation failed with 3 errors; no test executed.
```

These three fixture sites format a SHA-256 digest array with `{:x}`. The
production FastEmbed code already converts digest bytes to hex explicitly;
the fixture repair adopts that convention without changing hash content or
adding a dependency.

The same invocation reported6 warnings in surrealdb-core,38 in surrealdb,20
in UAR's library and1 in the auxiliary seed binary. The repair batch includes
only those emitted sites under the repository zero-warning rule. No suggested
`cargo fix`, lint, formatter or independent check was run.

State, runtime and lead released their separate source ownership and all three
handoffs are collected. Task9.6 is source-complete at waypoint3184, plan80.
The renewed saved-source boundary is recorded in
`lifecycle-verification-repair-inventory-2026-10-21.md` and its JSON file.
Only the same approved embedding command will rerun at this boundary.

The native candidate command remains held: its approved32cb3bcf source pin
predates the current uncommitted production repairs. The dependent Boss package
remains held until a matching candidate exists. Neither the published image nor
the old source pin certifies the new saved bytes.

## 3. Same approved embedding command — attempt2 FAILED

The source batch completed before this rerun. The saved-source inventory records
29294 file entries, aggregate
`sha256:9eabe7d62c1c6a1647d7385d94256c83324b718d068f0abd48f1975198c9551e`.
The exact embedding command above was rerun with only the recorded Cargo PATH
repair. Its live session is91450. Compilation progressed beyond the previously
failing fixture dependency preparation; no test result has been observed yet.
The process survived the interrupted conversation turn; the same handle was
resumed rather than starting another attempt. Compiler CPU time advanced on
direct process observation, so quiet output alone is not classified as a hang.

The current captured compiler diagnostic is one unused `OnceLockExt` import at
`vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/method/mod.rs:12`.
It is a warning, not a fatal diagnostic. The latest operator scope excludes
nonblocking cleanup; no further warning-only repair is initiated. The running
attempt's output is retained in the attempt2 log and must be updated with its
actual terminal outcome.

The operator continuation permits only finishing the current repair cohort and
this same approved embedding command. Canonical waypoint3185 / plan81 records
that narrower scope. Other ready commands are held. The implementation/evidence
checklist is `lifecycle-plan-implementation-evidence-checklist.md`; it explicitly
records the absent finite shutdown observation budget and requested design
decision. No new budget API or other broader production change is implemented.

Terminal result: exit101. Actual output:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 14m 32s
running 1 test
local_embedding_lifecycle_reuses_backup_across_two_cycles_and_refuses_an_equal_dimension_other_model ... FAILED
tests/embedding_lifecycle_failover.rs:569:10:
save owner-scoped knowledge base: knowledge requires a verified principal or the explicit anonymous boundary
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.67s
```

The digest compilation failures did not recur. The new failure is fixture owner
identity rejected by the existing knowledge authorization boundary. State owns
a targeted fixture-only repair through the verified-principal convention;
production authorization must remain unchanged. No new test or warning-only
cleanup is authorized. Renew the source inventory after handoff before the
same approved embedding command reruns. If this same authorization failure
recurs after its targeted repair, stop affected execution and report it.

## 4. Same approved embedding command — attempt3 PASS

Task9.7 is source-complete at waypoint3189 / plan82; state writer handoff
collected. The fixture now uses the production-verified principal key/context
through all existing storage and query assertions. No production guard was
changed. The new source boundary is
`lifecycle-verification-principal-repair-inventory-2026-10-21.md` and its JSON
companion. Only after that inventory is authored will the identical embedding
command above rerun. There is no new gate or warning-only repair.

The renewed inventory contains29298 saved-file records, aggregate
`sha256:3902617d5158f3556f5206c69513022fc3783f95c193638764f0fcca951d878a`.
The identical command ran serially with the same installed Cargo PATH. Its
terminal exit was0. Actual output:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 40.81s
running 1 test
test local_embedding_lifecycle_reuses_backup_across_two_cycles_and_refuses_an_equal_dimension_other_model ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.35s
```

Captured output: `lifecycle-verification-output-embedding-2026-10-21-attempt-3.log`.
Both targeted failure categories are resolved in this execution. No passing
command is repeated or broadened. One nonfatal unused-import warning remains
at the SDK method module; it was not changed under the narrowed repair scope.

The passing existing test exercises real local inference through primary and
backup generations1→2→3→4→5, durable owner-scoped ingestion/search, preserved
embedding binding, actual different-model equal-dimension inference, explicit
HTTP409 `embedding_space_mismatch`, unchanged KB binding/document IDs and
successful compatible querying after refusal. Its cleanup assertion requires
supervisor release plus persistence scope release and engine-close
acknowledgement. It does not exercise immediate engine reopen, every watcher,
whole-server teardown, concurrent credential migration, sidecar security,
Windows, process-restart hydration or the whole frozen embedding contract.

The fixture and previously integrated source repairs remain uncommitted in the
current worktree; the published32cb3bcf image/pin excludes them. No new Docker
run, native candidate, package or other gate was started. The same approved
embedding command is now terminal; no verification process remains owned by
this execution. All other commands stay held under the latest instruction.
