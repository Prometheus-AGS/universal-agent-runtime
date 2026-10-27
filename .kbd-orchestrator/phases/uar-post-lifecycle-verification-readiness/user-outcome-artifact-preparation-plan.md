# Artifact preparation proposal — not execution authority

Active phase: `uar-post-lifecycle-verification-readiness`, implementation.
Operator gate CLOSED. This source-only document names candidate preparation
commands and missing producers; it is not an executable packet or permission.
No install/build/import/check/service/browser/provider command ran.

Skills consulted: `prometheus-rust-workspace` for feature/target and serialized
Cargo boundaries; `agent-team-handoff` for source/owner/evidence transfer. The
operator lock overrides executable skill steps. Root owns final inventory,
amendment and exact authorization. Others share this checkout.

## Source-derived commands and their effects

Working directory is the repository unless stated. These are argv proposals,
not commands issued during this task:

| Purpose | Known argv | Output and limit |
|---|---|---|
| Native server | `cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend` | Default dev-profile `target/debug/universal-agent-runtime`, unless adopted Cargo configuration overrides target directory/target. Root must freeze actual absolute artifact location and feature command. `server-full` includes minimal/Surreal; postgres-backend is separately required for both fixture stores. No stub or test binary is implied. |
| Shipped browser UI | `pnpm -C frontend build` | Manifest script is exactly `vite build`; current config writes `static/` and empties that output directory. Preserve any needed prior artifact first in an approved new staging snapshot. This builds source, not UI acceptance. |
| TS SDK library | `npm --prefix sdks/typescript run build` | SDK script is `tsup src/index.ts --format cjs,esm --dts --clean`; produces library dist, declarations and invokes its compiler. It does not produce emitted acceptance examples. |
| TS example declaration check — later verification only | `npm --prefix sdks/typescript run examples` | Script is `tsc -p tsconfig.examples.json --noEmit`; no emitted JS. Must not be represented as the TypeScript protocol client producer. Not part of artifact preparation unless separately named. |

The repository root build merely delegates to the frontend. Root BDD prepare
also builds the local entity workspace, validates the static bundle, builds a
stub and invokes bddgen; it is NOT an appropriate hidden preparation shortcut.
No `build:mac:arm64` script was found in the two inspected root/frontend manifests;
this task does not invent a Boss build command or waive any external milestone
obligation. Native `build.rs` compiles committed protobuf inputs, copies the
catalog snapshot and generates model code; Cargo build is a compiler/build gate
including build scripts, not inert source inspection.

## Installed dependency drift and staged exceptions

Known installed-vs-locked drift: A2UI installed0.10.4 vs locked0.12.0, local
entity React4.2.0 vs locked umbrella/core3.0.3, Playwright installed1.61.1 vs
locked1.62.1. Earlier snapshots/receipts remain historical. No build against this
installed tree can establish the locked closure merely because manifests agree.
A2UI and entity API source reconciliation handoffs record the minimal repairs.

The lead must request explicit staged exceptions, naming actual executable
paths, cwd, argv, effects, unfinished derived items and output locations:

1. Lock-based dependency materialization into an owned staging checkout/prefix,
   not silent mutation of operator node_modules. Candidate pnpm operation is
   `pnpm install --frozen-lockfile --ignore-scripts` from the selected workspace
   root. It is NOT finalized: root must resolve root/frontend lock authority,
   pnpm11.15.0 executable identity, workspace linking to local4.2.0 versus pinned
   registry3.0.3, exact staging prefix and lifecycle/native-addon needs. Ignoring
   scripts may leave required tools unusable; approved dependency lifecycle
   commands must be enumerated separately rather than implicitly enabled.
2. TS SDK dependency materialization uses its own `sdks/typescript/package-lock.json`;
   it is outside root `pnpm-workspace.yaml` globs. Candidate isolated operation
   is `npm --prefix sdks/typescript ci --ignore-scripts`, subject to exact owned
   staging location/tool and required lifecycle reconciliation. A root pnpm
   install does not establish this SDK's closure.
3. Explicit native, frontend and SDK library build commands from the table,
   serially, with captured command/exit/output identities. These are build
   exceptions while derived artifacts do not yet exist; no implementation item
   or unbuilt artifact is renamed complete to avoid approval.
4. A distinct TS SDK acceptance-example emitter remains unresolved: both the
   examples script and tsconfig.examples.json specify noEmit. The composer
   expects a real `typescript_sdk_entry` executed by Node. Root must identify or
   author a concrete emission/bundling producer and final entry path before
   proposing its argv; library dist alone does not satisfy the example binding.
5. Browser binary/video-encoder preparation requires exact already available
   pinned artifacts or separately approved download/install argv. No Playwright
   browser install command, free-port probe or service preflight is implied.
6. Private-input preparation is separately effectful: existing program argv is
   `[frozen_python, absolute_prepare_private_inputs_py, absolute_private_config_json]`.
   Actual source-bound configuration, credential-copy authority and paths remain
   explicit inputs. This step creates fresh secrets; it is not read-only compose.

No proposed exception includes tests, lint, formatter, typecheck, package audit,
Docker, service readiness, browser launch, models or acceptance indirectly.
Those require their own exact later approval. Build failures stop their stage;
no automatic repair/retry allowance is granted here.

## Installed immutable snapshot / closure producer proposal

The final composer requires complete groups named `native`, `python`, `node`,
`python_sdk`, `typescript_sdk`, `rust_sdk`, `grpc`, `playwright`, `browser`,
`video_encoder`, `classifier_wasm`, `local_embedding_assets`. Each group needs
`complete:true`, exact files and a frozen manifest `{kind,files:[{path,sha256}]}`.
A file entry hash is not the complete transitive dependency/resource closure.

No dedicated complete installed-snapshot producer was identified in the inspected
source. Do not fabricate a script or executable argv. Proposed bounded producer
needs a separate source implementation/ownership assignment before use:

- Copy approved built outputs and all transitive runtime dependencies/resources
  into a new exclusively owned absolute snapshot root. Capture symlinks plus
  resolved bytes and fail unresolved links; never relabel mutable workspace
  links as registry installed packages. Include full static index/assets and
  browser resources, interpreter/library modules, native dependent libraries,
  Playwright Node tree, SDK executable entries and dependency trees, WASM and
  embedding model/tokenizer/config resources.
- Determine native/interpreter transitive closure through source/platform-defined
  tooling. Exact commands are unresolved and require explicit approval; neither
  a guessed `otool` probe nor ambient path traversal is authorized by this prose.
- Produce content-addressed manifests, source→build command→artifact receipt,
  exact package versions/lock provenance and resulting immutable snapshot entry
  paths; seal only after complete capture. Freeze all bytes in final inventory,
  including manifests. Never include generated adoption/signature/fixture outputs
  in their own signed source preimage.
- Match the composer contract: installed server path equals artifact path; build
  receipt carries exact source identity, artifact identity/path, successful exit
  and approved command equal actual command. Freeze the static closure too.
  Root must approve this full closure, not infer it from a `complete` boolean.

The uncomfortable thing: the actual closure producer and TS example emitter are
not yet known executable paths. A fully runnable final preparation packet cannot
be claimed from this document. Root must resolve these plus staged installation
policy, final absolute tools/output paths, native profile and refreshed source
inventory before presenting commands to the operator. Signed authority/adoption
and resource reconciliation remain independent later prerequisites.

Evidence: source reads of root/frontend/SDK manifests, Cargo features/build.rs,
workspace globs, SDK tsconfigs, Vite config/plugin, composer closure/build checks,
private-input README and historical receipts. Only this document and the paired
consumption reconciliation were authored. Source identity hashing, installs,
builds and product verification were not run. No acceptance or phase completion.
