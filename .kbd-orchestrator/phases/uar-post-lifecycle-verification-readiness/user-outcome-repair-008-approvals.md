# A11 / bounded repair008 / corrected P0.005 approval

PROPOSED, UNSIGNED, NOT ADOPTED, NOT EXECUTED. A10 binding12 remains current. Product verification is CLOSED.

The uncomfortable thing: the corrected SDK lockfile has not passed a locked build. P0.004 passed native/TypeScript, then failed the Rust SDK lockfile gate; its three later stages never ran. There is no complete immutable artifact closure, executable final fixture or user-outcome acceptance. Original preparation has13/17 recorded passes; none is silently promoted to final acceptance.

## Exact bounded repair

Only `sdks/rust/Cargo.lock` changes, one line: local `prometheus-parking-lot-core` version0.3.0to0.4.0, matching the already vendored crate and authoritative pins. All901 package records/dependency lists are retained apart from that local version; registry versions/checksums, manifests, root lockfile, vendor and pins are unchanged. No new guard or production behavior.

Patch `sha256:28dafcfdda51faad51e0e7063093949d4760ad8f4151cdcb6ba93f3d14262f03`: `user-outcome-repair-008.patch`. Before bytes reconstructed from this substitution match frozen source008.

Before `sha256:ae80e1b692e43e8fab6b56475c43be137b966b1c62c7f30f67790c002e14c3f9`; after `sha256:6bed587ee451e31d3c2002ec9247e24af16f9f2fd0d9de6e4fcc854fbeadf791`.

One authorized non-compiling offline authoring command exited0:

```bash
cargo update --offline --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/rust/Cargo.toml --package universal-agent-runtime
```

The raw offline result also proposed three unrelated registry downgrades (inotify and two tokio_with_wasm records). Those exact original records were restored by source editing, leaving only the one local version change. No Cargo retry, build, test, check, formatter, compiler/import probe, review gate or network fallback ran. Exact raw receipt, before bytes and final diff are in the worker handoff; the raw authoring hash and final source hash are distinguished in provenance. Successful authoring is not compilation success.

Source009 `sha256:b8114c31f92bb25420e35df8d4751bc7be5b7906eb946aaa420bbb978f012d5b` binds29,002 regular files and195 retained symlink/deletion entries. Only this lockfile differs from source008. Native KBD revision494/plan132; task2.18 source complete,2.9 in progress. All writer handoffs collected; no active writer. Required derived artifacts/fixture/authority, final acceptance/reviews/supporting gates and archive/reflection remain pending. This inventory is source-only, not full-phase implementation completion.

## Exact amendment and limits

Proposed contract `8aca00bff66b22108aea2ea78cbe634c8d08208ff35e1565f64a6941c47c4c4b` supersedes A10 `dc575f8ec79e2cbfea88276d46646f63294744eeae4f2889aa50479550f74b24` only after actual signature/adoption. All111 scenario obligations, including14 predecessor obligations, remain identical:0 added/modified/removed.

Raise only cumulative repair cap7to8. Source repair008 was explicitly authorized; all8consumed/no ninth. Keep original epoch, all prior consumption/overruns/UNKNOWN history and failed receipts;4,000 cumulative emitted host calls,2,400 conservative active minutes,no-progress3 and all existing token/model/embedding/output/file/tree bounds remain. Keep separate USD10 Cohere/USD10 OpenAI embedding caps with no transfer. P0 makes no provider calls/spend; OpenAI remains unused.

Accounting008 cutoff `2026-10-25T12:56:31.065705+00:00`: 2470 unique dispatches and1216.564938 conservative active minutes across47 sessions. Later activity and operator waits accrue under only previously signed idle exclusions. Historical pre-epoch effort remains UNKNOWN, repair floor8; no reset or new idle exclusion.

Future exact signing/adoption commands, only after explicit A11 approval:

```bash
ssh-keygen -Y sign -f /Users/gqadonis/.ssh/id_ed25519.pub -n prometheus-kbd-scope-amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-008.amendment.json
bash /Users/gqadonis/Projects/prometheus/prometheus-skill-system/shared/scripts/kbd-contract-adopt.sh .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-008.proposed.json --supersedes contract-adoption-uar-post-lifecycle-verification-readiness-dc575f8e --amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-008.amendment.json --signature .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-008.amendment.json.sig
```

Use the existing registered public key via SSH agent; no private-key read, trust registration or credential modification.

## Corrected P0.005 and native rebind exception

Retain ten materialization passes, frontend source007 PASS and TypeScript source008 PASS. Selected frontend and TS byte inventories are `user-outcome-retained-frontend-004.json` and `user-outcome-retained-typescript-005.json`. Source008 native PASS remains historical. No install, browser download/launch, frontend/TS rebuild, image pull/inspect, version/import/service/provider probe or added preflight. Resource mismatch stops, with no fallback rebuild.

Explicitly approve one cached native build after prior PASS, because the unchanged producer requires its actual native-stage receipt to bind the current whole-source denominator. The independent SDK lockfile changes source009 even though the native command is unchanged. This is the named exception to the normal rule to rerun only the failed gate; it is not inferred from old permission. Source inspection cannot manufacture the current-source receipt. Retain prior native passes and their time; do not call them fresh source009 success.

Create exclusive005 parent/evidence0700; preserve001/002/003/004 scopes/logs/failures. Reuse002 tool/resource/model/package trees, native/SDK/grpc caches and unrun private guest root under SAME002 build-owner.lock. Retain original static backup. No source/production lockfile rewrite; only the declared private guest Cargo.lock/bindings can be generated.

Draft `user-outcome-preparation-005-config.draft.json` stays executable=false without approval hash. After actual A11 adoption, activate only admission/projection fields and downward-clamped limits; write private005 config/approval/toolchain-only environment and invoke producer once:

```bash
/Users/gqadonis/miniconda3/bin/python3 /Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/prepare_artifacts.py /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-005/artifact-preparation.config.json
```

Five serial commands, once each inside producer, first failure stops:

### P0.7

```bash
/Users/gqadonis/.cargo/bin/cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{"CARGO_TARGET_DIR": "UAR_PREP_NATIVE_TARGET"}`.

Effects: native build and build scripts; Cargo dependency/cache network if required; Cargo JSON selects actual server bin; native target isolated from later Cargo commands; Explicit cached native rebuild exception after a previous PASS: bind current whole-source identity to native receipt required by unchanged artifact producer. No broader gate or frontend rebuild..

### P0.10

```bash
/Users/gqadonis/.cargo/bin/cargo build --locked --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/rust/Cargo.toml --example user_outcome_acceptance --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{"CARGO_TARGET_DIR": "UAR_PREP_RUST_SDK_TARGET"}`.

Effects: compile SDK example and build dependencies; do not run it; Cargo JSON selects example executable; independent SDK lock.

### P0.11

```bash
/Users/gqadonis/.cargo/bin/cargo test --locked --features server-full,postgres-backend --test user_outcome_grpc --no-run --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{"CARGO_TARGET_DIR": "UAR_PREP_GRPC_TARGET"}`.

Effects: explicit no-run test-harness compiler exception and build scripts; Cargo JSON selects user_outcome_grpc executable; no test execution.

### P0.12-resolve-guest

```bash
/Users/gqadonis/.cargo/bin/cargo generate-lockfile --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/Cargo.toml
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{}`.

Effects: Resolve exact-direct-pinned guest dependencies into only the new private generated Cargo.lock; no repository source or product lock mutation; Private Cargo.toml/src/WIT copied from frozen guest templates before this stage.

### P0.13-build-guest

```bash
/Users/gqadonis/.cargo/bin/cargo-component component build --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/Cargo.toml --locked --release --target wasm32-unknown-unknown --target-dir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/cargo-target
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{"CARGO": "UAR_PREP_CARGO"}`.

Effects: Compile/componentize current eligible-skill keyword classifier; emit private generated bindings and component; Explicit internal rustc --print sysroot and installed wasm32-wasip1 prerequisite inspection; missing target fails, no automatic rustup install; bindings.format=false prevents implicit rustfmt; no component instantiation or UAR strategy test.

Keep --locked on the Rust SDK build. No separate command followed by implicit producer rerun. Exact baseline environment refs, tool hashes, selected resource roots, outputs/closure mappings and forbidden inherited flags are in the JSON packet/frozen draft. Cargo dependency network/build scripts are disclosed. gRPC uses --no-run only. Guest resolution/build writes its private root, with only the already named internal sysroot/installed-target inspection; no automatic rustup installation or formatter.

P0 original14,400second ceiling is retained. All prior001/002/003/004 consumption remains in receipts;004 consumed49.644078seconds from carried12,115, leaving a conservative12,065 whole seconds for005. Deduct every005 setup/metadata second before launch and clamp further to remaining global limits. Native prior attempts592.804484+589.496367+4.493167seconds remain consumed; shared stage cap6,013seconds keeps their cumulative total below original7,200seconds and tightens other queued stages. Failed Rust SDK attempt0.706070seconds remains consumed. Output/file/tree bounds unchanged.

Stop on first failure, changed source/resource, ownership mismatch, unknown/exhausted bound; exact-owned process cleanup only and evidence retained. No ninth repair, automatic retry, product verification, services/provider/model/embedding calls, publication, archive or reflection. If anything fails, discuss an immediate repair versus backlog with the operator BEFORE edits or retry. Success still requires actual derived fixture/private-input authority, refreshed installed inventory and separate final V approval.

## Exact approval text

> Approve A11 signing/adoption and corrected P0.005 exactly in this packet, including the cached native source-identity rebind after prior PASS, locked Rust SDK build and three unrun stages. Raise only repair cap7to8, all8consumed/no ninth. Preserve every prior consumption entry, other limits and USD10 per-provider caps. Sign and adopt, then run the five commands once through the serial producer with first-failure or unknown/exhausted-bound stop. Retain completed frontend/TS/materialization results. No further repair or retry without discussing immediate fix versus backlog; no product verification, service/provider call or publication.

Fresh authority is required by the signed first-failure contract and AGENTS.md operator lock after source edits. The native rebind exception is explicit. No previous approval grants execution of this new packet.
