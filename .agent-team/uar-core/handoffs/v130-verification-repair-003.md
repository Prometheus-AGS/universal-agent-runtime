# v1.3.0 verification fixture repair003 handoff

Phase: `uar-post-lifecycle-verification-readiness`; Execute. Operator verification
gate: CLOSED. This bounded source repair follows the operator's explicit request
to fix `main_build_boot_identity_mismatch` and prepare an amendment permitting
one additional repair. The signed authority still has total cap2 until amendment;
proposed amended total cap3, consumed3 after this repair, with zero automatic
repairs remaining. No new time/call epoch or additional automatic repair is
created. This handoff authorizes no gate, signature, adoption or publication.

## Files and exact source identity

- `scripts/post_lifecycle/main_boundary.py`: 8354 bytes; SHA-256 `cd467d5a54b211106b525378da71436972af38709876550af8eb368ad773ec25`.
- `tests/post_lifecycle_main.rs`: 39894 bytes; SHA-256 `f388b145501c816ab9baea45c779e5f495b93f1f7e68ea9e92a4d05023ab8e23`.
- `.agent-team/uar-core/handoffs/v130-verification-repair-003.md`: this handoff.

Only the two assigned source files and this new handoff were authored. Other
writers share this checkout; existing edits were preserved. The lead owns all
manifest, contract, spec, inventory, canonical-state and memory authoring.

## Observed failure and bounded implementation

Retained packet003 `source-build-boot.json` records built SHA-256
`7be53484dcc48e988982cc5f02a6d2c572f67a7715bf372a8bfae7d39127203d`.
The reported main launch refused with `main_build_boot_identity_mismatch`.
The previous guard did not retain the expected/observed comparison. The lead
inspected declared, root-resolved and candidate paths and found agreement. The
runner previously hashed mutable Cargo debug output before invoking Cargo test.
Cargo replacement during test preparation is a plausible cause, not a proved
upstream behavior or observed root cause. The repair avoids relying on that
mutable output for later launches without weakening identity requirements.

`main_boundary.py` preserves the exact BUILD and TEST argv. It derives the sole
build output from `projectRoot/target/uar-post-lifecycle-main/debug/universal-agent-runtime`
and the sole frozen destination from `evidenceRoot/artifacts/universal-agent-runtime`.
The manifest must contain those exact absolute `buildOutputPath` and
`candidatePath` values, and `processes.main.argv[0]` must equal the frozen path.
These checks are before build execution. Fresh append-only evidence creation
and private umask remain in place. After successful BUILD and before TEST, it
hashes build output, creates a mode0700 artifacts directory, copies bytes in
process from the build output to an exclusively created (`xb`) candidate, grants
mode0700 execution, and hashes the frozen file. It retains both paths and
SHA-256 values plus `freezeOutcome` in `source-build-boot.json`. A differing
copy is retained as BLOCKED and refused before TEST. The existing
`builtMainSha256`/`UAR_BOUNDARY_BUILT_SHA256` stay bound to the pre-copy build
hash. `UAR_MAIN_CANDIDATE` names only the frozen copy. No copy executable,
extra Cargo command, retry, process, probe or provider call was introduced.
A partial copy or filesystem error remains a blocking error and leaves private
evidence in place; it is not retried or overwritten.

`post_lifecycle_main.rs` writes an exclusive
`main-build-boot-identity-{launch}.json` receipt before the existing main mismatch
guard. Each comparison retains expected candidate path, observed argv0, expected
build SHA-256, observed executable SHA-256, each equality result and PASS/BLOCKED
identity outcome. This identity outcome is not case or phase acceptance. The
unchanged guard still refuses either path or SHA difference with
`main_build_boot_identity_mismatch`; it does not accept a post-test replacement
hash. Receipt-write failure also stops launch through existing Result propagation.

Repair002 startupTransportRetry, owned-child polling and cleanup were preserved.
Scenario/oracle definitions, process start counts, restart semantics, request
budgets, teardown criteria, SIGTERM ownership and timeout behavior were not
changed by this repair.

## Scope and guards

The frozen identity and exact-path guards address the reported main build/boot
mismatch at the executable trust boundary. Exclusive creation and private
permissions preserve the existing append-only private evidence boundary. The
pre-copy/frozen comparison ensures the copy cannot silently change accepted
identity. Comparison receipts address the missing differing values in packet003.
No unrequested guard or adjacent cleanup was added.

## Commands and unverified claims

Every command was classified as source inspection or authorized implementation/
documentation writing. Source inspection used `cat`, `sed`, `rg`, and `rg --files`
to read waypoint, source, retained packet003 build identity, AGENTS, dependency
pins/manifests, relevant gotchas, Rust workspace/router/best-practices/async
instructions and error guidance, Python patterns and prior repair handoff.
`apply_patch` authored the two source edits. In-process Python read source bytes,
computed the hashes above and exclusively wrote this handoff; it did not import
or execute project code.

No test, Cargo command, build, compiler probe, lint, format, typecheck, readiness,
packaging, policy validator, automated acceptance review, git diff check,
signing, adoption, commit, push or publication ran. No new tests were authored.
Compilation, copy execution, identity receipt generation, startup, all scenario
behavior and teardown remain unverified under CLOSED. Mandatory independent
acceptance review is deferred under the operator lock.

## Lead continuation

Bind `buildOutputPath` to the absolute project target path above. Bind
`candidatePath` and main argv0 to the absolute fresh
`evidence/main-boundary-v1.3.0-004/artifacts/universal-agent-runtime` location.
Freeze the amended whole-phase source inventory after collecting every writer's
handoff. Reconcile waypoint and complete task ledger, prepare the unsigned
single-extra-repair amendment and exact proposed gate commands, and obtain
explicit named operator approval before any gate. Subsequent implementation
edits close the gate again. A future failure must be retained and surfaced;
there is no further automatic repair allowance.
