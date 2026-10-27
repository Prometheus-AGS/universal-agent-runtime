# A9 / bounded repair006 / corrected P0.003 approval

Status: PROPOSED, UNSIGNED, NOT ADOPTED, NOT EXECUTED. Current A8 binding10 stays authoritative. Product verification is CLOSED.

The uncomfortable thing: the repaired source has not compiled. Ten preparation steps passed, but the native build failed exit101 and six later stages never ran. No fixture, complete artifact closure or user-outcome PASS exists yet. This packet requests preparation only.

## Exact source and repair

Whole-phase source inventory: `sha256:9be55f1940937db4475b976fc204f868c568c3169e911707d005d6abf0338517`; 29,002 regular inputs and195 symlink/deletion metadata entries. Source007 is a source snapshot, not implementation-complete or executable acceptance authority. Inventory positions canonical revision480/plan128; task2.16 source complete, task2.9 in progress. Prior source006 and all failed receipts remain intact.

Patch: `sha256:6bf3d0b5f894c6b172e41ddd11a9f193ba543278b40e08fee9507263c4858ce9` in `user-outcome-repair-006.patch`. Exactly three production substitutions across two files. Before-file bytes were reconstructed from the edits and match frozen006 hashes; current source007 differs from006 in these two files only.

| File | Before SHA256 | After SHA256 |
| --- | --- | --- |
| `src/uar/persistence/scoped.rs` | `1a4f12411911e50df0f85fa6aca52e4a3caf267c6a57f44a3984e96622059954` | `0022538c568ab20cdc9141e1aae2e758a9a95b0db2680f094fb1f8995e2bb17e` |
| `src/llm/liter_driver.rs` | `9668d3478041841d480cfb073168a96a5e7cfae1068a942caf44e879d73ce0e9` | `0cea5cda674f06cfd78881bb127197170f377d2746fe5b579d5468322cb206b2` |

Two nonexistent RAG fingerprint paths now use the existing canonical domain type. SHA256 serialization now uses existing bytewise two-digit lowercase hex encoding. No dependency, guard, retry, fallback or unrelated production edit was added. No compiler/build/test/format/check or review-driven verification ran for this repair. Independent Rust review remains pending under the final gate.

## Amendment to approve and sign

New contract: `589e132bbff66585d8d67aaee97dfeeeb76d21181353f3fbc942b7e334bb02b9`.

Supersedes only A8 `fddca102edb4a2eaa38dab39fafd653011cad00f7a225fd1af1398dc47b09ae6`. All111 scenario obligations, including14 predecessor obligations, are retained with0 added/modified/removed scenarios. Historical assertions remain historical; missing prior evidence does not become PASS.

Only cumulative repair cap changes5to6. Repair006 is already source-authored under your explicit instruction; all6 are consumed and no seventh is authorized. Keep original epoch, all calls/time/overruns/UNKNOWN history, 4,000 cumulative calls,2,400 conservative active minutes,no-progress3, all model/embedding/output/file/tree bounds and independent USD10 Cohere/USD10 OpenAI embedding caps. No transfer. P0 makes no provider call or embedding spend; OpenAI remains unused.

Accounting004 cutoff `2026-10-25T11:20:25.631156+00:00`: 2390 unique emitted host dispatches, 1120.474362 conservative active minutes across45 sessions. This is a cutoff floor, not a fresh balance; subsequent activity and operator waits accrue with only previously signed idle exclusions. Repair floor6. No new idle exclusion.

Future signing/adoption commands (do not execute until explicit A9 approval):

```bash
ssh-keygen -Y sign -f /Users/gqadonis/.ssh/id_ed25519.pub -n prometheus-kbd-scope-amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-006.amendment.json
bash /Users/gqadonis/Projects/prometheus/prometheus-skill-system/shared/scripts/kbd-contract-adopt.sh .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-006.proposed.json --supersedes contract-adoption-uar-post-lifecycle-verification-readiness-fddca102 --amendment .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-006.amendment.json --signature .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/user-outcome-exit-contract.repair-006.amendment.json.sig
```

This uses the already registered public signing key through the SSH agent. No private-key read, signer registration, credential change, publication or implicit product-gate opening is included.

## Corrected preparation scope

Carry forward the ten PASS materialization receipts from `evidence/user-outcome-preparation-002/stop.json`; do not reinstall packages, redownload browsers, repull/reinspect images or run added probes. Retain exact002 resource trees/tool hashes, local models, native dependency cache, unrun SDK/grpc targets and unrun guest root. Use the same002 build-owner.lock for every build. Preserve002 artifacts/logs and prior-static backup.

Create exclusive003 parent/evidence directories and private config/approval/toolchain-only environment. Rebind exact source007 and the seven existing approved stage shapes to new003 artifact output. The frozen draft is `user-outcome-preparation-003-config.draft.json`; it deliberately has executable=false and no approval hash. After actual A9 adoption, activate only its admission fields, exact projection approval and a downward-clamped time budget. No source/tool/tree replacement or unlisted command is authorized; mismatch stops.

Invoke the producer once after metadata sealing:

```bash
/Users/gqadonis/miniconda3/bin/python3 /Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/prepare_artifacts.py /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-003/artifact-preparation.config.json
```

The producer runs exactly these seven commands once, serially, stopping at the first failure:

### P0.7

```bash
/Users/gqadonis/.cargo/bin/cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{"CARGO_TARGET_DIR": "UAR_PREP_NATIVE_TARGET"}`.

Effects: native build and build scripts; Cargo dependency/cache network if required; Cargo JSON selects actual server bin; native target isolated from later Cargo commands.

### P0.8

```bash
/Users/gqadonis/.volta/bin/pnpm -C /Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend build
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Child references: `{}`.

Effects: Vite product build; empties and rewrites repository/static; no test/lint/typecheck script wrapper.

### P0.9

```bash
/Users/gqadonis/.volta/bin/node /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/node_modules/typescript/lib/tsc.js --project /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/tsconfig.examples.json --noEmit false --outDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/.acceptance-artifacts --rootDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript`. Child references: `{}`.

Effects: explicit compiler/emit exception; emits example and src JavaScript/declarations; Node entry .acceptance-artifacts/examples/user-outcome-acceptance.js; preserve SDK type:module package.json and node_modules.

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

All exact baseline environment references, forbidden inherited variables, outputs, resource roots/tool hashes and closure mappings are in the retained producer draft and commands JSON. The seven stages run inside the producer, never separately before an implicit producer rerun. Cargo build scripts/dependency network remain disclosed; no test execution. The gRPC command compiles with --no-run only. Guest resolution writes only the declared private guest lock/bindings, and component tooling may inspect its explicitly named installed target/sysroot; no automatic rustup or formatter.

Original P0 total ceiling14,400seconds is not reset. Retain prior001 1.350480196seconds and corrected002 1,584.103835seconds of wall consumption through failure. The tighter carried ceiling leaves12,813 whole seconds maximum for003; deduct all003 metadata/setup wall time before producer launch and clamp further to remaining global active/call limits. Previous native attempt592.804484seconds is retained; shared producer stage cap6,607seconds keeps the native cumulative limit below7,200 and tightens queued stages to the same cap. File/output/tree bounds stay unchanged. Unknown/exhausted allowance stops before dispatch.

Stop on first failure, changed source/resource hashes, unknown/exhausted budget or ownership mismatch. Perform only exact-owned process cleanup after failure; preserve evidence/cache. No seventh repair, automatic retry, product scenario, service startup, provider/model/embedding call, standalone preflight, publication or final V is authorized. Successful P0 must be followed by the actual derived fixture/inventory and separate exact final-V approval. Canonical acceptance/archive/reflection remain pending until all exit criteria pass.

## Exact approval text

> Approve A9 signing/adoption and corrected P0.003 exactly in this packet. Increase only cumulative repair cap5to6; all6 consumed, no seventh. Retain all prior consumption, other limits and USD10 per-provider embedding caps. Sign and adopt, then run the failed native build and six unrun stages once through the serial artifact producer, carrying the ten passed materializations and stopping at first failure. No automatic retry, further repair, product verification, service/provider call or publication.

Fresh approval is required by the adopted contract's first-failure boundary and AGENTS.md's operator verification lock after source edits. This request does not claim the full-phase implementation boundary is complete; it requests the exact preparation exception needed to finish task2.9.
