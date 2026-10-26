# Artifact preparation and installed closure snapshots

SOURCE AUTHORED ONLY. Verification gate CLOSED. No invocation, module import,
syntax check, test, installation, build, Docker, service, browser or model ran.
This new program replaces the missing producer identified in the phase artifact
preparation proposal; it does not turn that historical proposal into approval.

`prepare_artifacts.py` is a single serialized future-approved owner. Import has
no I/O. Future invocation is precisely `[frozen_python, absolute_program_path,
absolute_private_config_path]`. It has no install/discovery command, default
build, shell, service startup or probe. Actual argv/cwd/environment references,
resources, source hashes, outputs and finite limits must be configured and
separately approved. The operator must explicitly approve this whole future
program invocation and its contained commands/effects before it runs.

Python patterns and security-review guidance were consulted: explicit data
boundaries, named environment references, no eval/shell, bounded I/O and private
failure retention. Executable skill checks remain deferred under the lock.

## Configuration contract

Private config and approval are absolute, regular nonsymlink, owner-owned files
with no group/other permissions, at most64MiB. Config contains:

| Field | Exact meaning |
|---|---|
| schema_version / executable | 1 / true only in later actually approved inputs; no executable config is shipped here. |
| approval | `{path,sha256}` for the private actual approval bytes. |
| output_root | New absolute exclusive output root beneath an existing private owner-owned nonsymlink parent. Existing scopes are never removed or reused. |
| build_owner_lock | Shared absolute private lock path outside output_root, identical for every preparation sharing build targets. Actual exclusive nonblocking flock is held through commands/capture. |
| source_files | Nonempty sorted unique rows containing exactly `{path,sha256}`. These are the final composer source denominator, not generated artifacts or authority. |
| source_identity | SHA256 of compact sorted-key canonical JSON of source_files, same as composer preimage. Actual source bytes checked before commands, after every stage and after capture. |
| limits | Positive integers: total_seconds<=14400, stage_seconds<=7200, output_bytes<=67108864 per stage, file_count<=1000000 including directories/links, file_bytes<=17179869184, tree_bytes<=137438953472 aggregate. Root chooses tighter approved bounds. |
| stages | Nonempty ordered array, at most32 unique stage IDs. Each has id, argv, tool_sha256, cwd, environment_env and outputs. |
| native_stage | ID of the stage whose actual emitted server must become server_binary. |
| trees | Explicit nonoverlapping resource roots, each with id and either path+tree_sha256 for existing bytes, or output naming an actually produced output. Built tree may also give an ancestor path containing that output (needed to retain SDK package/module layout). |
| closures | All12 names below, each a nonempty list of selected tree IDs. |
| installed | Mapping of actual composer installed keys to `{tree,relative}`. Relative paths cannot escape their tree; resolved installed files must stay inside the new snapshot. |
| os_baseline | `{excluded:true,description:<explicit adopted system baseline and exclusions>}`. No all-machine-byte claim. |

Approval content must equal `{operation:"prepare-uar-acceptance-artifacts",
operator_approved:true,projection_sha256:<sha256 of whole config except approval>}`.
This avoids signing its own preimage. A JSON true is NOT a cryptographic signature
or operator instruction; the lead's external signed/adopted preparation exception
must bind exact program/config/approval bytes and effects. The program checks
matching bytes but does not independently verify a signer.

Stage `argv[0]` is an absolute executable whose actual bytes must equal
`tool_sha256`. `cwd` is an actual absolute nonsymlink directory. No shell parses
argv. `environment_env` maps child variable names to explicitly named existing
parent references. No ambient environment, PATH, provider secret, profile or shell
option is inherited unless explicitly mapped. The operator freezes toolchain,
cache, loader, HOME/PATH and lifecycle needs in those mappings; missing values
fail rather than use defaults. Never map raw provider keys into build stages.
Environment names alone enter receipts, not values.

Each stage output has a unique id and either an absolute expected path, or
`cargo_target`+`cargo_kind` to select the unique actual executable in Cargo JSON
compiler-artifact stdout. Cargo must therefore be configured with
`--message-format=json-render-diagnostics`; no guessed hashed gRPC executable
name or executable search is used. Outputs must exist and cannot be symlinks.
Commands stop on the first failure, timeout, output overflow or missing output.
No retry, cleanup replacement or further stage runs after failure.

## Source-backed producer argv recipes

These recipes must be expanded to approved absolute executable/entry/output
paths in config, not copied as unresolved strings into an executable packet.
The commands themselves are never inferred/executed by this program.

- Native, cwd repository: frozen Cargo argv `build --locked --bin
  universal-agent-runtime --features server-full,postgres-backend
  --message-format=json-render-diagnostics`; output selector
  `{id:"server",cargo_target:"universal-agent-runtime",cargo_kind:"bin"}`.
  Current server-full includes minimal/Surreal; Postgres must be additive.
- Frontend, cwd frontend: frozen Node plus frozen installed Vite CLI JS path and
  `build` (or exact frozen pnpm `-C frontend build`); manifest build is Vite only.
  Expected output absolute repository/static directory. Current Vite config
  empties this directory; its mutation must be in approval. Include static index,
  hidden `.vite/markdown-engine-graph.json`, every asset and referenced resources.
- TypeScript examples, cwd the approved staged sdks/typescript package root:
  frozen Node, actual frozen SDK TypeScript `lib/tsc.js`, `--project`, absolute
  tsconfig.examples.json, `--noEmit`, `false`, `--outDir`, explicit new emitted
  directory, `--rootDir`, absolute staged SDK root. Optional declaration flags
  are unnecessary: inherited declaration/declarationMap emission is allowed.
  This is a real emitting argv, not `npm run examples`, which forces noEmit.
  Expected entry is `<outDir>/examples/user-outcome-acceptance.js`; emitted
  `<outDir>/src/*.js` retains actual `../src/index.js` imports. Preserve the SDK
  ancestor package.json (`type:module`) and node_modules resolution layout in
  the selected built tree. Without that layout plain emitted .js is not claimed
  runnable. No test code is executed; this compiler invocation still requires
  an explicit build/compiler exception.
- Rust SDK, cwd repository: frozen Cargo `build --locked --manifest-path`,
  absolute sdks/rust/Cargo.toml, `--example user_outcome_acceptance
  --message-format=json-render-diagnostics`. Output selector cargo_target
  user_outcome_acceptance, cargo_kind example. Its independent manifest/lock
  and dev-dependency build effects must be in preparation; Cargo build does not
  run the example.
- gRPC acceptance client, cwd repository: frozen Cargo `test --locked
  --features server-full,postgres-backend --test user_outcome_grpc --no-run
  --message-format=json-render-diagnostics`. Output selector cargo_target
  user_outcome_grpc, cargo_kind test. This compiles a test harness and build
  scripts without running its test. Explicit approval must name the no-run
  compilation exception; it is not general test permission.

Source basis: root Cargo.toml native/test target declarations; independent
sdks/rust/Cargo.toml and example source; SDK package.json + tsconfig.examples.json
and emitted-import `.js` source references. The inspected installed TypeScript
`frontend/node_modules/typescript/lib/_tsc.js` option declarations define noEmit
as boolean, outDir/rootDir as file paths; parseOptionValue consumes literal false
for boolean options. This is source inspection, not a compiler probe. The SDK
currently has no installed TypeScript package at its own node_modules path;
actual locked SDK installation/tool path remains an explicit input. No version
or executable hash is fabricated.

## Closure and snapshot output

All12 closure names are required: native, python, node, python_sdk,
typescript_sdk, rust_sdk, grpc, playwright, browser, video_encoder,
classifier_wasm, local_embedding_assets. Every existing tree must match its
adopted tree_sha256 before copying. Its canonical tree hash includes sorted rows:
directory `{relative,kind}`, link `{relative,kind,target}`, regular file
`{relative,kind,sha256,bytes,executable}`; root relative path is `.`. Built trees
are captured only after their bound stage/output exists; current actual bytes
are recorded, never prefilled with invented output hashes.

Recursive capture includes hidden files, directories and symlinks. Regular file
bytes are copied and checked against the captured identity; the original tree
is read again to refuse mutation during copy. Special files, oversized inputs,
missing resources and overlapping roots fail. Every link chain must resolve
inside the explicitly selected roots; external/dangling links fail. Links are
rewritten relatively into the new snapshot and provenance records original link
and final snapshot target. Cross-tree links add their target trees transitively
to each relevant closure. This preserves Python/Node/browser/resource layout;
it never follows arbitrary machine trees to invent a dependency denominator.

Closure `complete:true` means complete capture of the explicitly adopted tree
set and its selected link targets. The operator/lead must establish that those
trees represent actual application runtime dependencies under the separately
named OS baseline. Native libraries, interpreter stdlib/modules, Node SDK
dependencies, Playwright's exact locked library/browser/video resources,
classifier WASM and all embedding/tokenizer/config bytes require explicit roots.
No native dependency probe or package install runs implicitly.

`output_root/snapshot/preparation.json` supplies the composer fragment:
installed_artifact and build_receipt refs; installed absolute snapshot entry
paths; all12 closures `{complete:true,files:[absolute regular paths],manifest:
{path,sha256}}`; manifests exactly `{kind,files:[{path,sha256}]}`; source identity
and denominator; excluded OS baseline and symlink provenance. Native receipt
binds the actual native stage command/approved command, successful exit,
source_identity and copied artifact identity/path. The source server binding
must be the actual emitted native-stage server, not an unrelated tree file.
Composer owner confirmed these shapes. Final fixture inventory still merges
these real file/manifests with source, private-input, authority and generated
configuration references; this producer does not adopt or sign that fixture.

The snapshot's regular files are sealed mode444/555, directories500; logs stay
outside it under a private directory and are mode400 after successful stages.
This is filesystem immutability against ordinary writes, not WORM storage or
protection from the owner changing permissions. Log output is bounded, raw and
private; it is never printed and may contain compiler diagnostics/paths. Failure
retains private logs, partial snapshot, ownership and failure receipts; no
snapshot PASS or complete.json is emitted. No scope is automatically deleted.
The configured common build_owner_lock acquires an actual exclusive nonblocking
flock for this invocation (macOS/Linux). Every preparation sharing targets must
use that same adopted path; unrelated tools that ignore the lock remain the
lead's single-writer responsibility. The inert lock file persists, but its OS
lock is released on exit; no stale PID is treated as current ownership.

## Remaining actual preparation inputs

Root must supply/adopt frozen final source denominator, exact approval/config,
new private output/staging roots, actual toolchain executables and hashes,
explicit environment mappings, all expected outputs and12 approved resource tree
sets/hashes, OS baseline, installed entry mappings and appropriate finite bounds.
Dependency installation remains separately approved: existing A2UI/entity/
Playwright drift must be repaired to the locks before selecting these roots;
SDK has its own package-lock and is outside root workspace. This program does
not install, infer tariff/consumption, sign, compose or execute the packet.

The uncomfortable limitation is that a copied closure can only be as correct as
its explicitly approved resource denominator. Omitting an actual dependency is
not repaired by checking a hash or writing complete:true. Current source is
unexecuted and no runtime portability/durability/compiler result is asserted.
Only this new program and README are changed by this assignment; parent owns
phase inventory and exact preparation exception. Source003 stays historical.
