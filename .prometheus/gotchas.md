# gotchas

Append-only. Dated entries. Mark superseded entries; do not delete them.

## 2026-08-09
- Initialized by prometheus-context-bootstrap.

## 2026-08-09 — `.prometheus/` was gitignored, and git reported the tree clean

**Symptom.** During routine worktree cleanup, a worktree queued for deletion held
~48 knowledge files — including UI/UX migration completion records for the active
KBD phase — that existed nowhere else. `git status` reported the tree clean.

**Root cause.** `.gitignore` carried a blanket `.prometheus/` rule, commented as a
"machine-local knowledge cache, not shared project content." That is the opposite
of the directory's purpose: it is the estate's append-only memory, and the base
rules describe it as git-tracked for exactly this reason.

**Why it was nearly invisible.** Ignored files do not appear in `git status`. A
clean status is evidence about tracked files only. Deleting the worktree would
have destroyed ~1.2M across ~226 markdown and jsonl files silently, with no diff,
no warning, and no recovery path.

**Fix.** Removed the blanket rule. The only exclusion is
`.prometheus/knowledge/.prompt-snapshots/` — hash-named LLM snapshots, roughly 37M
of the directory's ~38M, regenerable on demand.

**Prevention.** Never add `.prometheus/` to `.gitignore`. Before deleting any
worktree, check it for `.prometheus/` content absent from the origin repo. Treat a
clean `git status` as proof about tracked files and nothing else.

Carried forward from the pre-migration `AGENTS.md` (2026-08-09), where it lived
inside Appendix C of the v3 base rules.

## 2026-08-09 — vendored submodules still carry v3, and a nested AGENTS.md re-imports it

**Symptom.** The root context was migrated off Base Rules v3 to a 1,396-word
managed region. Four files inside vendored submodules still contain the full v3
constitution — 45 rule IDs each:

| Words | File |
|---|---|
| 3133 | `crates/prometheus-skill-system/AGENT_BASE_RULES.md` |
| 5041 | `crates/prometheus-skill-system/skills/imported/prometheus-entity-management/AGENTS.md` |
| 5297 | `crates/prometheus-skill-system/skills/imported/prometheus-entity-management/CLAUDE.md` |
| 3133 | `frontend/packages/prometheus-entity-management/AGENT_BASE_RULES.md` |

**Effect.** Claude Code loads a nested `AGENTS.md`/`CLAUDE.md` on demand when a
file in its directory is read. Working anywhere under
`crates/prometheus-skill-system/skills/imported/prometheus-entity-management/`
pulls **5,041 words of v3** into context beside the new root rules — the
two-constitutions state the migration exists to remove, reintroduced by path.
`AGENT_BASE_RULES.md` is not auto-loaded by filename, so those two are inert
unless something reads them explicitly; the nested `AGENTS.md` and `CLAUDE.md`
are the live hazard.

**Scope.** Both are git submodules, not repo content:

```
crates/prometheus-skill-system            (v1.7.0-9-gc2556154)
frontend/packages/prometheus-entity-management  (@prometheus-ags/entity-graph-tauri@3.0.0-alpha.0-10-gbbd6824)
```

**Do not fix them here.** Editing submodule content from the consuming repo
either fails to persist or creates a detached-HEAD diff that the next
`submodule update` discards. The fix belongs upstream in
`prometheus-skill-system` and `prometheus-entity-management`, each of which
needs its own migration off v3.

**Until then.** When working inside those subtrees, expect v3 to be resident
alongside the root rules and treat the root `AGENTS.md` as authoritative on any
conflict. Note the overlap in the session log rather than acting on whichever
constitution loaded last.

## 2026-08-08 — wasmtime / wasmtime-wasi major skew broke the certified profile

**Symptom.** `cargo check --locked --no-default-features --features server-full`
failed with 9 errors in `src/uar/runtime/skills/wasm_runtime.rs` and
`src/uar/runtime/wasm/sandbox.rs`. The messages named
`wasmtime_internal_core::error::error::Error` — a type that appears nowhere in
this repo — so they read as unrelated to any recent change. `sandbox.rs` was
byte-identical to `main` and still failed.

**Root cause.** `Cargo.toml` declared `wasmtime = "47"` beside
`wasmtime-wasi = "46"`. `wasmtime-wasi` re-exports `wasmtime` types, so the skew
put two distinct `Linker<T>` types in one build and `add_to_linker_sync` stopped
type-checking. `Cargo.lock` carried both 46.0.2 and 47.0.3.

The skew was introduced by merging dependabot #218 (wasmtime 46 -> 47) while its
companion #220 (wasmtime-wasi 46 -> 47) sat blocked on a lockfile conflict. Half
a coupled pair landed. `wasm-runtime` is in `server-full`, so this broke the
certified profile on `main`.

**Fix.** Bumped `wasmtime-wasi` to 47 and regenerated the lockfile once across
the change (PR #236). Verified: `cargo check --locked --no-default-features
--features server-full --all-targets` exits 0.

**Prevention.**
- These two crates share a major version. Never merge a bump to one without the
  other; a partial bump is worse than no bump. Pinned in `versions.toml` as
  `wasmtime-lockstep`.
- `MERGEABLE` on a dependabot PR means git can merge it, not that it compiles.
  Compile before merging a dependency change.
- Four independent dependabot lockfiles do not compose. Cherry-picking them in
  sequence produced a `Cargo.lock` matching no coherent resolution, which
  `--locked` rejected outright. Batch coupled bumps and run
  `cargo update --workspace` once.

The same shape recurred hours later in the frontend: npm group PR #234, labelled
"minor-patch", moved `@assistant-ui/react` 0.14.26 -> 0.15.4 in the lockfile and
removed the `useMessage` export, breaking `pnpm typecheck` on `main` at 7 call
sites. Group dependency PRs can carry breaking changes regardless of the label.

## 2026-08-09 — the skill-description budget is ~41x over, machine-wide

**Symptom.** Skills that exist and test fine do not fire. The failure presents as
"the skill is installed, its name is listed, it never auto-triggers."

**Measured, not estimated.**

| Scope | Skills | Description chars |
|---|---|---|
| repo `.claude/skills` | 56 | 13,075 |
| user `~/.claude/skills` | 916 | 250,681 |
| plugins `~/.claude/plugins` | 1,294 | 388,309 |
| **total** | **2,266** | **652,065 (~163,000 tokens)** |

`skillListingBudgetFraction` is 0.02, so the budget is ~4,000 tokens at a 200k
window. That is **~41x over**. `verify.sh` independently measured 2,267 skills
and ~163,254 tokens (40.8x) — the one-skill delta is scan timing.

**Mechanism.** Claude Code reserves a fraction of the context window for *all*
skill descriptions across every scope. Past that budget it silently drops the
lowest-priority ones: the skill keeps its name, the description vanishes, and
auto-triggering dies. Eviction ranks by `usageCount x 0.5^(days/7)`, so a newly
installed skill scores zero and goes dark first — a catch-22, since it can never
be auto-invoked to earn a score.

**Two measurement traps, both hit during this investigation.**

1. **Wrong denominator.** Counting only `.claude/skills` gives 56 skills and
   ~13k chars, which looks like 10x headroom. It is 2% of the real total. The
   budget is machine-wide; measure `~/.claude/skills` and `~/.claude/plugins`
   too. Also note `find <dir> -name SKILL.md` and `ls <dir>` disagree — the
   latter counts only top-level directories and undercounts badly.
2. **Folded scalars.** `grep -m1 '^description:'` on a `description: >` block
   returns the empty remainder of that line, reporting a 1-char description for
   a skill that actually has 328. Use a real YAML parser. Three repo skills were
   misreported this way before the retraction.

**Raising `skillListingBudgetFraction` does not fix a 41x multiple.** Going from
0.02 to 0.03 buys 2,000 tokens against a ~159,000-token overage. The real fix is
plugin gating — split the estate into domain marketplaces so only the active
profile's descriptions load — plus a name-routing meta-skill, since invoking by
name works even when a description has been dropped.

**Not fixed here.** That work is estate-wide and belongs in its own session.
This entry is the measurement and the mechanism, recorded so the next person
does not re-derive it from the wrong denominator.

## 2026-08-10 — the sidecar HTTP token is not full runtime shutdown

**Observed behavior.** Cancelling the caller-supplied HTTP
`CancellationToken` stops the Axum listeners, but the `server-full` future can
remain alive awaiting the A2A gRPC task. The existing SIGINT/SIGTERM handler is
what cancels the separate runtime cancellation root.

**Persistence consequence.** Awaiting only the HTTP serve loop is not enough to
make an in-process SurrealKV cold reboot deterministic. SDK-owned handles can
retain the directory lock after the HTTP listener has drained.

**Working test pattern.** Use a dedicated child process for each boot. Cancel
the caller token and prove HTTP has stopped, then send SIGTERM through the
unchanged signal path and await normal child exit. Process exit releases any
remaining SDK-owned file descriptors before reopening the same path. Do not
claim that the HTTP token alone shuts down the full runtime.

---

## 2026-08-13 — an installer that renames on collision cannot also report success

**Symptom.** Skills were current on disk yet unreachable. A Codex session
blocked on `deep-research`; `~/.claude/skills/deep-research` held an unrelated
April stub (1 file, 4,582 bytes) while the real 13,519-byte skill sat at
`prometheus-deep-research`.

**Root cause.** When something it does not own holds a skill's canonical name,
`install-plugin-generation.js` diverts to `prometheus-<name>` (`:930`, `:987`)
and `targetDestination` (`:1038`) re-derives the same fallback, so
`verifyTargets` (`:1045`) validates the renamed path. The run then prints
"Verified immutable generation installed to all supported user targets."
19 skills were affected across 14 targets.

**Why it was nearly invisible.** Sixteen of the nineteen were symlinks that
*resolve* — into a source checkout rather than the installed generation. Any
check asking "is this a symlink that resolves?" calls them healthy;
`artifact-refiner` served four-month-old content across six targets on exactly
that basis. Verification followed the rename by construction, so no run could
detect it.

**Fix.** Collisions are reported and exit non-zero;
`scripts/verify-skill-install.js` asserts every skill at every target (163 × 14)
with the denominator printed, requires symlinks to resolve into the active
generation, and hashes copy targets file-by-file. It runs as the install's last
step. Six failure modes are observed failing in
`scripts/tests/verify-skill-install.test.mjs`.

**Prevention.** A completeness claim without a denominator is not evidence. Ask
of any green check: *what would this print if it had failed?* Full record:
`.prometheus/postmortems/2026-08-13-skills-not-installed-at-canonical-names.md`.

---

## 2026-08-14 — disabling sccache does not disable the external Cargo build directory

**Observed behavior.** Cross-target checks run with `RUSTC_WRAPPER=` and
`SCCACHE_DISABLE=1` still spent long silent intervals in uninterruptible I/O on
`/Volumes/my-passport`. Process state showed `cargo` as `U`/`Us`, and build
artifacts resolved under `/Volumes/my-passport/cargo-build/...`.

**Mechanism.** The user-global Cargo configuration independently sets both
`rustc-wrapper = "/opt/homebrew/bin/sccache"` and
`build-dir = "/Volumes/my-passport/cargo-build/{workspace-path-hash}"`.
Clearing the wrapper disables compiler caching, but it does not override the
external build directory. Those controls solve different problems.

**Working practice.** Retain the execution session and poll it instead of
mistaking an empty output interval for a compiler deadlock. Record literal
target environment values. For Android, the installed NDK uses the
API-suffixed `aarch64-linux-android35-clang`, and the pre-existing
`native-tls` graph needs a target OpenSSL sysroot. Do not repair either machine
precondition in UAR source.

## 2026-08-14 — `CryptoProvider::install_default` cannot identify the installed provider

**Observed failure.** A provisional guard compared the pointer returned by a
failed `rust_crypto::DEFAULT_PROVIDER.install_default()` call with the
RustCrypto static. The identical-RustCrypto control passed, but an AWS-LC-first
process was also accepted; the structured-conflict assertion failed.

**Root cause.** `jsonwebtoken` 11.0.0 delegates `install_default` to
`OnceLock::set(default_provider)`. On failure, `OnceLock::set` returns the value
the caller attempted to install, not the provider already stored. The installed
provider getter is crate-private. The returned pointer therefore always
describes the attempted RustCrypto value and proves nothing about the process
owner.

**Working rule.** UAR owns first installation and caches only its own successful
RustCrypto acquisition. Any provider installed earlier—including RustCrypto—
returns a structured conflict. Do not reintroduce pointer comparison unless a
pinned `jsonwebtoken` API publicly exposes the installed provider identity.

## 2026-08-15 — same-handle reconstruction is not a SurrealKV restart proof

**Observed defect.** A scoped-skill durability test built a second service over
the same live `Arc<SurrealDbProvider>` and called it a restart. Independent
review rejected the claim.

**Working rule.** For cold-restart assertions, launch a child process for each
boot against one temporary SurrealKV path. Process exit must release the old
provider before the next process opens it. A new service over one live provider
proves rehydration only.

## 2026-08-15 — compatibility must be observed at the behavior boundary

**Observed defect.** The durable skill model first dropped unknown legacy agent
binding IDs, then a repair made GET return them without making later-loaded
skills obey them during matching.

**Working rule.** When preserving an API contract, test the downstream behavior,
not only the response shape. For agent skills, set a binding before load, load a
selected and non-selected skill, and observe matching. Conversation and explicit
durable agent records remain more specific than the compatibility fallback.

## 2026-08-15 — skill provenance must survive both reads and writes

**Observed defect.** `provider_id` looked usable in the database but was rebuilt
from a filesystem provider that mixed API-created and configuration-managed
files. The first repair fixed cold reads but left the storage provider willing
to write any source into the API namespace. Old dynamic copies could also race
real configuration files by directory iteration order.

**Working rule.** Enforce provenance at the filesystem write boundary and test
upgrade residue explicitly. For duplicate IDs, configuration beats dynamic
regardless of traversal order. A clean-directory cold-reload test alone does
not prove the migration is safe.

**Evidence lesson.** A required `error!` call in source is not an observed log.
Install a test subscriber, run the exact test with `--nocapture`, and retain the
literal level, message, fields, and passing result.

## 2026-08-19 — JSON Schema validity is not refiner-state integrity

**Observed defect.** A malformed constraint entry containing iteration metadata
passed the permissive JSON Schema, while checkpoint history and registry
identity had drifted from the active artifact. Early receipts also summarized
Tier 0 instead of retaining the chronological checks run after each edit.

**Working rule.** Validate refiner artifacts semantically as well as
structurally: exact constraint IDs, iteration sequence, checkpoint references,
active/history identity, and registry artifact identity must agree. Retain the
actual chronological Tier 0 receipts so later checks cannot conceal an edit
that was never checked at its required point.

## 2026-08-20 — stable ID lists do not make entity projections reactive

**Observed defect.** The embedded SSE adapter updated an existing normalized
Knowledge entity, but the React view hooks projected `items` only when their ID
arrays changed. The graph held the new record while the screen continued to
render the old one. A browser retry, list reload, or screen-local cache bypass
would have hidden the source-package defect.

**Working rule.** A normalized view that exposes full entities must subscribe to
the snapshots behind its stable IDs. Test an existing-ID update in both the
current hook and its documented replacement, then repair the source package
instead of forcing consumer refreshes.

## 2026-08-20 — source-package builds must include declaration dependencies

**Observed defect.** BDD preparation invoked React-package `tsup` directly. On
the tested submodule pin it failed because `entity-graph-core`'s stale `dist`
did not declare `getGraphSyncStatus`, even though the source did.

**Working rule.** Build a source workspace package through its declared build
graph. For the entity-management React package, the Turbo dependency filter
must build core before React declarations; a direct leaf build depends on
checkout residue.

## 2026-08-20 — nested pnpm engines constrain consumer workspaces

**Observed defect.** UAR uses pnpm 11.15.0, but the nested entity-management
workspace admitted only pnpm 10.33.0. Dependency preparation stopped before the
product test even though the package code was compatible.

**Working rule.** Keep the repository's integrity-pinned default package manager
while expressing the tested consumer range in every enforcing workspace
manifest. A source submodule cannot claim consumer compatibility if its nested
engine rejects the consumer's package manager before build.

## 2026-08-20 — lock regeneration is not a minimum-delta proof

**Observed defect.** A frozen-compatible root lock candidate and two clean
regenerations agreed on dependency movements that were unrelated to the pinned
submodule manifest. Comparing only those generated candidates made the shared
drift look causal. Direct comparison with `HEAD` exposed the unrelated
config-array/minimatch and y-webrtc/ws movements.

**Working rule.** For a lock-only repair, classify every `HEAD`-to-candidate
mutation by the manifest change that caused it. Preserve unchanged-importer
edges even when a fresh resolver would legally select a newer version.

**Observed control.** After restoring the old y-webrtc edge, lock-only frozen
validation still passed but a clean full install failed because the changed
sync importer also required a direct ws 8.21.1 package record.

**Evidence rule.** Run both metadata-only and empty-dependency-tree frozen
installation. A receipt's displayed command must be capable of emitting every
recorded output line; prose describing an omitted parser or setup step is not a
replayable command.

## 2026-08-20 — each active pnpm workspace owns its lock boundary

**Observed defect.** The repository-root lock passed its checks while the
independently active `frontend/` workspace rejected frozen installation after a
pinned submodule manifest changed. Root-lock success did not describe the
nested command's dependency graph.

**Working rule.** Hash and frozen-test the lock belonging to the command's
actual pnpm workspace root. For a nested lock repair, classify every mutation
against the committed manifest or submodule-manifest edge that caused it and
preserve unrelated resolutions.

**Resolver lesson.** Two clean regenerations can agree on resolver drift that
is not required by the source change. Retain an exact candidate-to-raw patch and
compare against `HEAD`. Pnpm importer projections can also come from
auto-installed peers, so resolve evidence anchors against the manifest section
that actually declares the edge rather than assuming the importer key names it.

## 2026-08-21 — release gates are not deployment validation

**Observed defect.** A three-hour operational-resilience product test was put in
GitHub Actions because it built an installed archive and container, gated a
release, and uploaded evidence. None of those properties made it validate an
actual deployment. The run contradicted the standing deployment-only policy and
was canceled.

**Working rule.** Classify a check by the boundary it observes, not by its
workflow name or artifact. Only deployment execution, deployed configuration,
rollout, infrastructure wiring, and post-deployment health belong in Actions.
Run all product and release certification locally. Enforce the boundary with an
allowlisted local validator on every commit; if a plan says otherwise, revise
the plan before execution.

## 2026-08-21 — deleting a test workflow can delete the only test

**Observed defect.** Removing the prohibited security-audit workflow also
removed the only Dependabot-alert allowlist gate. The policy correction was
structurally right but behaviorally incomplete until the Rust, JavaScript, OSV,
Grype, and Dependabot checks had a local source-bound entrypoint.

**Working rule.** Before deleting a non-deployment workflow, inventory every
behavior it owns and map each required behavior to a checked-in local command.
The replacement must exist and pass its cheap contract check before the
workflow deletion is treated as complete.

## 2026-08-21 — worktree settings seeding can dirty immutable candidates

**Observed defect.** `scripts/worktree-new.sh` copied the operator's modified
`.claude/settings.local.json` over the same tracked path in a detached
candidate, so a newly created worktree was dirty before certification began.

**Working rule.** Seed per-tool settings only when the destination does not
track that path. If the candidate tracks it, preserve the committed copy; local
certification must start from a genuinely clean checkout.

## 2026-08-21 — do not default BuildKit's automatic platform arguments

**Observed defect.** A native Apple Silicon `docker build` selected an ARM64
Ubuntu base, but `ARG TARGETARCH=amd64` forced the Go, TinyGo, and Wasmtime
download branches to AMD64. TinyGo then failed at `dpkg` with an architecture
mismatch.

**Working rule.** Re-declare BuildKit's automatic platform arguments inside a
stage without a value. A default such as `ARG TARGETARCH=amd64` overrides the
detected target and can create a mixed-architecture image. Verify the fix with
the actual toolchain stage and its architecture probes, not a text check alone.

## 2026-08-21 — Docker contexts must exclude nested workspace outputs

**Observed defect.** The UAR image build copied the entity-management
submodule's macOS `node_modules`, package `dist` directories, and Turbo cache
into a Linux ARM64 builder. The first clean install then failed to load
Rollup's Linux ARM64 native package. Removing that residue also exposed that
the image build had been compiling the submodule's unrelated docs and examples
instead of installing its own frozen lock and building only UAR's two consumed
packages.

**Working rule.** Use recursive Docker ignore patterns for dependencies,
package outputs, and task-runner caches. Install each nested workspace from its
own frozen lock in the target container, then build only packages shipped by
the parent artifact. A package build that succeeds by replaying a host cache is
not portable evidence.

## 2026-08-21 — curl write-out consumed by Bash `read` needs a newline

**Observed defect.** The installed-candidate certifier completed provider
failure recovery, wrote a successful recovery response, and then exited 1 with
no diagnostic before MCP checks. Its `chat_request` helper emitted curl status
and latency without a trailing newline. Bash `read` assigned both values but
returned nonzero at EOF, and `set -e` terminated the script.

**Working rule.** When curl `--write-out` feeds Bash `read`, terminate the
format with `\n` and keep a focused contract check that observes `read` return
zero. A populated response artifact does not prove the surrounding shell
assignment succeeded.

## 2026-08-21 — observe tool failure at the event boundary

**Observed defect.** The installed-candidate MCP crash check used a
non-streaming chat response to decide whether a failed tool call had surfaced.
UAR emitted an unsuccessful normalized tool-result event and reconnected the
MCP transport, but the non-streaming endpoint intentionally retained only the
model's final `mcp-recovered` text. The certifier therefore labeled correct
event-level behavior as a replayed successful tool call.

**Working rule.** Certify a tool failure from the streamed tool-result event,
not from final assistant text or the request's overall HTTP status. Pair that
event with a fixture-side process trace that proves the failed call executed
once and the next independent call used a replacement process. Do not change
normal agent recovery semantics to satisfy a check observing the wrong layer.

**Correction from the same preflight.** The reconnect itself succeeded only in
the disposable filtered registry that handled the failed call. A later request
rebuilt its registry from the stale global service value and failed before
reaching the replacement process. Event-boundary evidence and shared reconnect
ownership are separate requirements; proving the former does not prove the
latter.

## 2026-08-21 — shared replacement pointers need authoritative configuration

**Observed defect.** The first MCP repair shared a replaceable service pointer
across registry views but left reconnect configuration on each view. An old
filtered view survived an A-to-B upsert, failed while using B, then reconnected A
into the shared slot. The first independent judge missed the reachable rollback;
the history-free critic blocked it with a concrete old-view sequence.

**Working rule.** When an asynchronous repair can replace shared state, keep the
repair inputs and a generation in the same shared ownership boundary. Snapshot
the generation, build outside the lock, and discard the result if newer
configuration won meanwhile. Test with a view created before A-to-B replacement,
not merely two calls using one unchanged configuration.

## 2026-08-22 — container release controls need cache-compatible build volume ownership

**Observed defect.** A focused Linux shutdown control initially entered the
full Dockerfile build after only three Rust source files changed. The layer
fingerprint invalidated the release target and began rebuilding the entire
SurrealDB, Wasmtime, OCR, and server-full graph. Switching to the existing
builder image avoided rebuilding the image layers, but its `/src` and target
fingerprints still required a 12-minute optimized compile. Compiler-cache and
build-volume mismatch, not unwritten shutdown code, was the delay.

**Working rule.** Implement and run Tier 0/focused tests in the configured
single-writer host target first. Build a source-only control commit, reuse the
matching architecture builder and its target volume, and perform the real
container boundary only after focused behavior passes. Keep Rust toolchains and
active target/cache volumes on the internal drive; do not move them or start a
fresh full container graph merely to obtain early feedback.

## 2026-08-22 — custom container ports can invalidate inherited health evidence

**Observed defect.** The first manual non-root shutdown control served on port
19161 while the inherited image healthcheck still probed production port 1906.
External readiness passed and the deadline behavior was correct, but Docker
recorded a healthcheck exec failure, so that run could not support a Docker
health claim.

**Working rule.** A focused container control that inherits a healthcheck must
use the healthcheck's configured port or replace the healthcheck explicitly.
Observe Docker `healthy` before initiating held work. Do not infer container
health from an external readiness probe alone.

## 2026-08-22 — a dated Docker ARG does not constrain a floating Cargo selector

**Observed defect.** The production Dockerfile installed
`nightly-2026-07-18`, but its backend build invoked `cargo +nightly`. On the
2026-08-22 ARM64 build host that selector resolved to `nightly-2026-08-22`, and
the locked `diskann-wide 0.54.0` dependency failed with three E0283 diagnostics.
The earlier Docker syntax check passed because it never compiled that path.

**Working rule.** Every Rust build stage must explicitly consume the declared
dated toolchain argument. Validate Docker default, repository channel, and
effective build argument together, pair the contract with a mismatched-channel
negative control, and complete a clean production-image build before handing a
release candidate back to certification.

**Follow-up.** `openspec/specs/gke-deployment/spec.md` still describes a Rust
stable/1.87 image build. That stale capability text is outside this release
child and must be reconciled in a separately planned spec change; it is not
evidence about the current production Dockerfile.

## 2026-08-23 — artifact-refiner workflow dispatch does not interpolate its payload

**Observed behavior.** After each successful filesystem checkpoint for
`uar-readme-estate-review`, the vendored artifact-refiner
`workflow-dispatch.sh` raised `JSONDecodeError: Expecting value` before it could
evaluate the artifact's empty trigger list.

**Root cause.** The Python body is introduced with a single-quoted heredoc but
contains shell variables such as `$EVENT_PAYLOAD` and `$STATE_FILE`. The shell
therefore does not interpolate them; Python receives the literal variable names
and attempts to parse `$EVENT_PAYLOAD` as JSON.

**Consequence.** Checkpoint and artifact state remain valid, but lifecycle
triggers cannot fire through this script. Do not treat dispatch as observed just
because the checkpoint command succeeded. No trigger was configured for this
documentation artifact, so the defect did not block its content gate.

**Ownership.** The implementation is in the pinned
`crates/prometheus-skill-system` submodule. Fix it upstream rather than creating
a detached submodule edit in UAR.

## 2026-08-23 — Pages environment branch policy is stricter than workflow triggers

**Observed behavior.** A manual `docs.yml` run on
`codex/uar-branded-documentation-site` assembled and uploaded the complete Pages
artifact, but the deploy job was rejected before execution. The
`github-pages` environment has a custom deployment-branch policy whose only
entry is `main`.

**Working rule.** A workflow's `workflow_dispatch` or branch trigger does not
grant deployment authority. Inspect the target environment's deployment branch
policy before planning feature-branch publication. Preserve the protection;
open the PR, obtain merge authority, and validate the deployment created from
the allowed branch instead of weakening the environment for convenience.

## 2026-08-23 — OpenSpec main specs and modified deltas have different structural rules

**Observed behavior.** Documentation archive first failed because the canonical
`dev-portal-2026` spec retained a delta-only `## ADDED Requirements` header. A
later archive then stopped because its `MODIFIED` requirement omitted a scenario
introduced by the now-applied foundation change. Both failures occurred before
the affected archive changed files.

**Working rule.** Canonical specs have one `## Requirements` section and never
retain delta operation headers. Before archiving successive changes that modify
the same requirement, refresh each later modified block against the current
canonical requirement and preserve every existing scenario by name. Apply
conflicting deltas in their planned chronological order and validate the touched
canonical spec after each archive.

## 2026-08-23 — Windows server-full cross-check requires target-scoped MSVC variables

**Observed behavior.** `x86_64-pc-windows-gnu` stopped in `ort-sys` because the
pinned distribution has no GNU Windows prebuilt. A direct cargo-xwin check then
set global `TARGET_CC` and `TARGET_CXX`, causing a macOS host build dependency to
invoke `clang-cl` against Apple assembly and headers.

**Working rule.** Cross-check Windows `server-full` as
`x86_64-pc-windows-msvc`. Generate the cargo-xwin environment, but pass only the
target-qualified compiler, linker, SDK, bindgen, and CMake variables to Cargo.
Host build dependencies must retain the host compiler. A successful cross-check
is compile-only evidence and makes no Windows service-runtime claim.

## 2026-08-23 — UAR liveness can pass while the native SurrealDB dependency is hung

**Observed behavior.** The installed UAR `/healthz` endpoint remained healthy
while `/readyz` and SurrealDB's own port-28000 health request timed out. UAR's
operational log stopped at its SurrealDB connection boundary after restart.

**Working rule.** Treat `/healthz` as process liveness only. When native
readiness stalls, probe the configured persistence dependency directly. Stop
UAR before restarting `ai.prometheus.surrealdb-native`, preserve the existing
RocksDB path, require the database listener/health response, and only then start
UAR and require `/readyz`.

**Limit.** The restart restored service, but it does not establish the internal
cause of the database hang. The preceding SurrealDB logs contained transaction
conflicts and one-minute query timeouts; do not report either as the root cause
without a separate diagnosis.

## 2026-08-23 — SurrealDB LaunchAgent restart requires an execution-state gate

**Observed behavior.** `launchctl kickstart -k` reported no command error but
left `ai.prometheus.surrealdb-native` in `xpcproxy` state with no port-28000
listener. HTTP health also returned once during an earlier restart before the
dependency stalled again.

**Working rule.** For this LaunchAgent, boot out UAR first, boot out SurrealDB,
and observe both labels absent from the user launchd domain. Bootstrap
SurrealDB and require both HTTP health and a successful WebSocket query before
bootstrapping UAR. Then require both UAR liveness and readiness. Do not treat
launchctl registration, a PID, or HTTP health alone as dependency recovery.

**Limit.** This is an operational recovery rule, not a diagnosis of the
underlying SurrealDB or RocksDB stall.

## 2026-08-23 — Registry package adoption invalidates workspace-only chunk rules

**Observed behavior.** After adopting the release build of Entity Management
3.0.2, the production frontend no longer emitted its dedicated entity vendor
chunk even though the application resolved the correct dependency.

**Root cause.** The Vite manual-chunk rule matched the former workspace source
path only. Registry packages resolve beneath
`node_modules/@prometheus-ags/...`, so the rule silently stopped matching.

**Working rule.** When a package moves between workspace and registry
provenance, verify the production asset graph as well as the lockfile. Chunking,
alias, transform, and source-map rules that inspect paths must recognize the
installed package layout explicitly.

## 2026-08-23 — A model inventory is not proof that the account can use a model

**Observed behavior.** The local OpenAI proxy advertised `gpt-5.4-nano`, but a
real completion request was rejected for the ChatGPT-backed Codex account.
`gpt-5.4-mini`, `gpt-5.4`, and `gpt-5.5` completed successfully.

**Working rule.** Use discovery only to populate candidates. Functional routing
evidence must come from an actual completion with the current account and
endpoint; never convert an advertised model ID into a usability claim.

## 2026-08-23 — Serial OpenSpec MODIFIED deltas must preserve prior archive scenarios

**Observed behavior.** Archiving `repair-session-configuration-entity-flow`
immediately after `adopt-entity-management-3-0-2` failed closed. Both changes
modified the same `Entity-management integration has one package boundary`
requirement, but the later delta omitted the dependency-drift scenario that the
earlier archive had just added.

**Working rule.** When serial changes carry MODIFIED deltas for the same
requirement, treat each archive as a new specification baseline. Before
archiving the later change, preserve every scenario added by preceding changes
unless the later proposal explicitly removes it. Let the archive guard reject
scenario loss; never bypass it with `--skip-specs`.

**Limit.** This is a spec-merge ordering rule. It does not imply that unrelated
changes should be combined or that an agent should edit generated KBD
projections by hand.

## 2026-08-24 — Frontend dependency adoption must reconcile both lockfile scopes

**Observed behavior.** The root workspace lockfile resolved Entity Management
3.0.3 and produced the correct production bundle, but `pnpm -C frontend exec`
stopped before Playwright because `frontend/pnpm-lock.yaml` still contained the
3.0.2 records. The hardened pnpm dependency-status check validates the nested
workspace before executing its command.

**Working rule.** A dependency pin in `frontend/package.json` participates in
both `pnpm-lock.yaml` and `frontend/pnpm-lock.yaml`. Reconcile and frozen-verify
both scopes. When an explicitly approved first-party release is inside the
minimum-release-age window, add the same exact-version exception to both
workspace files; never trust the complete lockfile or relax the global policy.

**Limit.** This dual-lock requirement applies while both workspace roots remain
in the repository. It is not evidence that other nested package locks require
the same application dependency.

## 2026-08-24 — An absent client projection is not an absent runtime route

**Observed behavior.** The Admin Agents list showed amber missing-model warnings
for two agents that completed inference by inheriting the configured
`kimi-for-coding/k3` system default.

**Root cause.** The Agents page classified status before loading provider
metadata, and the actively used provider-store hydration projected
`default_id` without the matching provider's `default_model`. The UI therefore
treated incomplete client state as proof that the runtime route was absent.

**Working rule.** Status indicators for derived cross-entity facts must remain
loading until their authoritative projection has completed. Hydrate all fields
used by the resolution predicate from one response, expose the result through a
typed domain hook, and distinguish confirmed absence from load failure. When a
status icon is not self-explanatory, make its existing row/control the hover and
focus tooltip trigger rather than nesting another interactive element.

**Limit.** This rule does not turn every asynchronous screen into a global
loading gate. It applies when an unloaded projection would otherwise be
presented as a confirmed operator-actionable defect.

## 2026-08-25 — Settings response row UUIDs are not provider identities

**Observed behavior.** The outer `id` values returned by
`/api/uar/settings/providers` changed after a clean LaunchAgent reinstall even
though the config hash, five provider keys, provider names, and durable
`data.id` values were unchanged and startup seeded zero providers.

**Root cause.** The SurrealDB adapter's `surreal_value_to_setting()` creates a
new in-memory proxy UUID for each loaded settings row. That outer UUID is not
persisted provider identity. Provider identity is `data.id` and is also encoded
in `key` as `provider.<id>`.

**Working rule.** For provider preservation checks, compare count plus
`data.id`/`key`, not the outer settings-row UUID. Treat a change in provider
keys, provider data, seed count, or config hash as the destructive signal.

**Limit.** This documents existing adapter behavior; the settings-route fix
does not change settings persistence or response payloads.

## 2026-08-25 — Native UAR readiness can exceed a fixed 30-second restart probe

**Observed behavior.** The macOS installer completed and LaunchAgent PID 31143
was running, but the first 30-second readiness loop ended before port 1906
responded. Service logs showed repeated SurrealDB connection refusals, followed
by a successful connection and normal listener startup roughly nine seconds
later. Health and readiness then returned HTTP 200 without another install.

**Working rule.** After a native restart, treat LaunchAgent state, dependency
logs, and bounded health probes as separate signals. Do not roll back or clean
build/worktree evidence merely because one fixed readiness window expires;
inspect logs and keep cleanup gated until the service either becomes ready or
exits with a terminal failure.

**Limit.** This is not permission for an unbounded wait. A dependency that never
recovers, repeated process exits, or a failed installed digest still blocks
cleanup and requires rollback or operator intervention.

## 2026-08-25 — Diff review packets can omit new untracked regression tests

**Observed behavior.** The first adversarial diff review reported that the required ProviderPanel regression test did not exist even though the file was present locally and its focused test passed.

**Root cause.** The deterministic diff packet was assembled from `git diff`, which does not include ordinary untracked files. Marking the new test intent-to-add made its content visible to the packet without committing it.

**Working rule.** Before dispatching a diff review, compare `git status --short` with the packet's diff file list. Ensure every relevant new file is present in the review artifact; otherwise a fresh judge is reviewing an incomplete change.

**Limit.** Intent-to-add changes index metadata. Do not stage unrelated user files, and do not treat packet inclusion as authorization to commit.

## 2026-08-25 — Matching requirement names do not prove a MODIFIED delta is synced

**Observed behavior.** The main `frontend-configuration-surfaces` spec already contained `Provider default models use bounded selection`, so a name-only check appeared to show the searchable-model delta was synced. The body still described the older non-searchable select contract.

**Root cause.** The sync assessment searched only for the requirement header instead of comparing the complete normative block and scenarios.

**Working rule.** Before archiving a MODIFIED delta, compare the entire requirement block from its header through its final scenario against the main spec. Ignore only formatting-equivalent trailing blank lines. Restore the active change and complete the merge if any body or scenario differs.

**Limit.** This does not require byte-for-byte comparison for intelligent merges that preserve additional valid scenarios; it requires semantic coverage of the full modified block.

## 2026-08-25 — Prompt-caching correctness requires tracing dispatch and usage end to end

**Observed behavior.** The Prompt Caching settings page returned 404, owner-scoped agent configuration returned noisy 404s for ordinary absence, Anthropic caching was applied unconditionally on some paths, and compatibility responses could report cache activity that no provider emitted.

**Root cause.** The settings namespace was never registered or mounted, policy fields stopped at partial contracts, production `LlmRequest` constructors bypassed a common strategy seam, the native Anthropic driver used an incorrect messages root and treated non-success upstream responses as successful dispatches, and compatibility code locally simulated cache usage.

**Working rule.** Treat a cache toggle as a control-plane policy. Inventory every production request constructor, carry an explicit provider identity through failover, make non-success provider responses fail dispatch, and derive cache metrics only from provider usage. Ordinary owner-safe absence uses empty 204; legacy frontend 404 handling is compatibility only.

**Deployment corollary.** A stricter startup invariant can break preserved native configs even when the current packaged default is correct. Native config migration must add missing loopback defaults without overwriting explicit operator values.

**Limit.** The repository MV3 entity-explorer submodule currently references three missing icon assets and therefore does not register its service worker when loaded exactly as checked in. Temporary restoration proved its connect/disconnect relay behavior, but the pinned submodule needs a separate upstream packaging change before exact-package extension certification can pass.

## 2026-08-26 — KBD apply positional IDs can duplicate pre-registered semantic tasks

**Observed behavior.** OpenSpec reported task `1.1` complete out of nine, while the canonical KBD projection reported ten tasks: the pre-registered semantic task `1.1` plus a new completed positional task `1` created by `kbd-apply begin-task`.

**Root cause.** The OpenSpec adapter emits positional IDs (`1`, `2`, …), but this change's KBD tasks were registered with section IDs (`1.1`, `1.2`, …). The apply driver treats a missing positional ID as a new canonical task and append-only runtime history has no task deletion command.

**Working rule.** When KBD tasks already use semantic OpenSpec section IDs, pass those semantic IDs to `begin-task` and `end-task`; the OpenSpec adapter's non-numeric fallback marks the matching checkbox text while the runtime transitions the existing task. Compare the OpenSpec task count with the canonical projection after the first boundary.

**Limit.** The extra completed task `1` in `fix-provider-settings-panel-width-responsiveness` cannot be removed through the typed runtime. OpenSpec remains 1/9 after task 1.1; the KBD projection is permanently offset at 2/10 for this change unless the runtime later gains an append-only supersession mechanism.

## 2026-08-27 — Adversarial packets need a change-owned inventory, not the ambient dirty tree

**Observed behavior.** The diff packet builder included unrelated tracked worktree changes, emitted no acceptance criteria, and omitted the new Playwright file, nested OpenSpec spec delta, and refinement artifacts. A judge could therefore review a large but incomplete candidate.

**Root cause.** Packet assembly used the ambient tracked Git diff and top-level change discovery. Ordinary untracked files and nested OpenSpec deltas were not part of that input model.

**Working rule.** Before dispatch, enumerate every change-owned tracked and new file, compare that list with the packet's `diff --git` headers, and inject the full proposal, design, tasks, nested spec, and blocking constraints. Reject a packet with unrelated files or empty acceptance criteria.

**Limit.** Mechanical packet correction is an auditable workaround, not a repair to the builder. A separate orchestration-tool change is required to make discovery correct by default.

## 2026-08-27 — Cargo filters are forwarded to the standalone BDD runner

**Observed behavior.** `cargo test --locked --no-default-features --features server-full governance` ran and passed all 21 matching library tests, then exited 2 when `tests/bdd.rs` rejected the positional `governance` argument.

**Root cause.** Cargo forwards a trailing test filter to every selected test executable. This repository's BDD executable accepts its own CLI contract rather than libtest's positional filter.

**Working rule.** Scope focused Rust filters to an explicit target: use `cargo test ... --lib <filter>` for library tests and `cargo test ... --test settings_persistence <filter>` for the Governance persistence/API integration target. Do not report the broad command as passing merely because the intended tests passed before the BDD runner failed.

**Limit.** This does not replace the exact unfiltered Tier 2 suite. That command remains the release truth and currently retains unrelated routing-evaluation failures.

## 2026-08-27 — Freeze the complete candidate before starting the final gate

**Observed behavior.** The exact Rust Tier 2 command was started while rollback and release-evidence artifacts were still being finalized. The operator stopped the run and required the repository's code-to-end, then test sequence.

**Root cause.** Verification was treated as an incremental defect-discovery loop after individual repairs instead of one end-of-work gate over immutable forward and rollback candidates.

**Working rule.** Finish production code, test code, fixtures, dependency reconciliation, rollback source, recovery procedure, and candidate commits first. Then run the complete ordered verification sequence once. If verification exposes a defect, stop the sequence, repair and refreeze both affected candidates, and restart the required gate from its beginning. Partial output from an interrupted command is never a receipt.

**Limit.** Read-only source inspection, candidate inventory, and artifact authoring are not verification. This rule does not permit skipping the final Tier 0–3, rollback, deployment, reflection, or archive evidence.

## 2026-08-28 — Database-live settings events can race runtime publication

**Observed behavior.** Independent final review found that a successful
Governance setting write could emit its database-live notification before the
coherent runtime gate published the accepted revision.

**Root cause.** The generic settings bus treated database mutation as the
notification boundary, but Governance authorization also has an in-process
publication boundary after persistence and cache update.

**Working rule.** For settings that control a live authorization gate, suppress
the generic database event and emit one explicit event only after the runtime
authority is published. Test both event ordering and the post-commit delivery
failure path.

**Limit.** This rule applies to live authorization/control-plane settings with
multiple publication boundaries. It does not require replacing ordinary
database-live notifications for unrelated settings.
## 2026-08-28 — A fail-closed rollback may normalize a durable Off preference

**Observed behavior.** A shared SurrealKV database with a seed-owned default row
booted Off under the forward candidate, On with mutation unavailable under
rollback, and still On after the forward candidate returned.

**Root cause.** The supported rollback candidate seeds/normalizes configuration-
owned values to its fail-closed On posture. API-owned rows carry `updated_at`
and are preserved; retaining the schema alone does not distinguish those cases.

**Rule.** Export and checksum the complete `governance.enabled` row before
downgrade. On return to forward, compare the ownership marker and value. Restore
only a seed-owned export when the current row is the known rollback
normalization; preserve API-owned or concurrent values.

## 2026-08-28 — Completion projections must not lead irreversible work

**Observed behavior.** The KBD plan claimed all 42 OpenSpec tasks were complete
while reflection, publication, verification, and archive were still unchecked.
Its progress projection also retained an obsolete certification blocker and a
false conclusion that no PR was required.

**Root cause.** Completion prose was updated from the intended terminal outcome
before the corresponding append-only lifecycle events and external publication
artifacts existed.

**Working rule.** Reconcile prose against the authoritative OpenSpec checkbox
count and remote branch/PR state at each phase boundary. Describe the observed
count, never the expected terminal count. Publication is complete only after
the remote refs and PR URL are observable; archive is complete only after the
change has moved to the dated archive path and strict validation passes.

**Limit.** Generated projections can still lag until the typed KBD transition
runs. A stale projection must be corrected before it is offered to a critic or
used as completion evidence.

## 2026-08-28 — Parent branch switches can silently reset a nested gitlink checkout

**Observed behavior.** The first Skill System follow-up PR recorded Surreal
Memory commit `f671111` instead of the patched `432eaa1`, even though the leaf
remediation had already been built and merged.

**Root cause.** Switching the parent worktree from its prior branch to
`origin/main` reset the shared nested submodule checkout to the gitlink recorded
by that parent commit. A subsequent parent commit therefore captured the reset
leaf without re-verifying its exact HEAD.

**Working rule.** Immediately before every parent gitlink commit, resolve and
print the nested repository HEAD, compare it with the accepted pin, and inspect
the remote PR's aggregate compare patch. A previously verified nested checkout
does not remain authoritative across a parent branch switch.

**Limit.** This concerns nested checkout state, not the accepted leaf commit.
The stale pointer was corrected by a normal follow-up commit before PR #75 was
merged and never reached parent `main`.

## 2026-08-28 — Offline extraction builds need bounded local build storage

**Observed behavior.** A fresh offline archive build reached final compilation
and failed with `No space left on device` while the global Cargo build cache and
sccache cache occupied the nearly full data volume.

**Working rule.** For archive acceptance, use a disposable empty `CARGO_HOME`
from inside the extraction so machine-global build configuration cannot redirect
intermediates into a shared cache. Check free space first and remove only the
task-created extraction after recording the artifact hash.

**Limit.** Disk exhaustion is not a source failure. The identical archive
subsequently built offline after the regenerable compiler cache was cleared.

## 2026-08-28 — A workspace audit does not cover an independently locked SDK

**Observed behavior.** Root and frontend pnpm audits were clean, but the final
`sdks/typescript` npm audit still resolved `nanoid` 3.3.16 through PostCSS and
reported GHSA-2v37-7h3g-55p8.

**Root cause.** The TypeScript SDK owns a separate npm lockfile, so the root and
frontend lock refreshes did not update its transitive dependency graph.

**Working rule.** Dependency-refresh certification must enumerate and audit
every independently locked package root. A root workspace audit is evidence for
that lockfile only, not for sibling npm projects.

**Resolution.** Refreshed the SDK lock within PostCSS's existing semver range to
`nanoid` 3.3.18, then observed a zero-vulnerability audit, four passing tests,
and successful CJS, ESM, and declaration builds.

## 2026-08-28 — Whole-worktree source packaging crosses the credential boundary

**Observed behavior.** The offline packager selected the entire working
directory and excluded only a short path list. A checkout containing ignored
`.env` or OpenTofu `terraform.tfvars` files would therefore copy them into the
release staging directory.

**Root cause.** Filesystem-wide selection treated tracked source and
operator-local state as the same packaging domain.

**Working rule.** Source archives must derive their input set from Git's tracked
index, including recursive submodule files, and then add generated vendored
dependencies inside the isolated staging directory. Never package a checkout by
enumerating everything and trying to blacklist every possible credential file.

**Resolution.** Changed the offline packager to consume the NUL-delimited
`git ls-files --recurse-submodules` set while retaining the existing explicit
exclusion for root `.claude/settings.local.json`.

## 2026-08-28 — Liter stream establishment is not completion collection

**Observed behavior.** A native `web_fetch` completed successfully, then the
second model call in the tool loop failed with `LLM stream start timed out after
15000 ms`. The provider had established the first response stream in about
seven seconds, while the post-tool completion exceeded the 15-second limit.

**Root cause.** UAR's Liter adapter collected every upstream chunk into a
`Vec` before returning the normalized stream. The orchestrator's stream-start
timeout therefore measured the whole completion rather than response-stream
establishment.

**Working rule.** Preserve an upstream owned stream through adapter boundaries.
Normalize and emit chunks incrementally; never collect a provider stream merely
to satisfy an ownership assumption without checking the dependency's current
lifetime contract.

**Resolution.** Liter 1.18.2 exposes a `'static` chunk stream. UAR now moves
that stream into its normalized wrapper and records full-call latency when the
upstream stream completes.

## 2026-08-28 — A2UI metadata is not proof of an A2UI surface

**Observed behavior.** Chat labeled an effective-policy artifact as an A2UI
surface while rendering the complete serialized policy JSON as one text node.

**Root cause.** The message store fabricated an A2UI profile, the custom-event
route handled individual frames without retaining surface lifecycle state, and
the display component manufactured a local Text/Column surface instead of
processing the supplied protocol messages.

**Working rule.** A display artifact is renderable A2UI only when it carries the
declared UAR profile, passes the production-version schema and catalog checks,
and is processed as an ordered, bounded surface lifecycle. Metadata labels must
never substitute for protocol validation.

**Resolution.** Runtime policy artifacts now emit production v0.9.1 messages;
the frontend accumulates bounded per-surface lifecycles and renders them through
the canonical processor and UAR surface package. Invalid source is disclosed in
a bounded diagnostic panel.

## 2026-08-28 — Tracing capture test can race under the full Rust suite

**Observed behavior.** The first full `server-full` run failed only
`retrieval_emits_decision_audit_event`; the exact isolated rerun passed, and a
complete rerun passed all enabled tests.

**Root cause.** The test's subscriber-local captured event can be affected by
parallel tracing tests. No A2UI code participates in this path.

**Working rule.** When this single capture assertion fails in a parallel run,
run it exactly once in isolation and then rerun the complete tier. Do not patch
unrelated RAG behavior unless the isolated test or full rerun fails.

## 2026-08-29 — Standard skill aliases may contain a selector in an ancestor

**Observed behavior.** The real `~/.agents/skills` tree contains top-level aliases
whose declared target is below a plugin path such as `.../current/skills/name`,
where `current` is itself a version selector symlink. Rejecting every symlink in
the target's ancestor chain discarded valid installed skills even though the
bounded target surface contained no followed links.

**Working rule.** For standard-skill top-level aliases, reject a direct final
target symlink and never follow links within the resolved target, but allow a
literal ancestor selector to resolve during canonicalization. Keep the scan
bounded to the root manifest, conventional `skills/` subtree, or immediate
manifest-bearing children of a flat collection.

## 2026-09-02 — kbd-new-child: runtime branch and file branch disagree on the parent

When the waypoint is in the "selected but not entered" state (path[] tail equals
childPointer), `kbd-new-child.sh` computes the parent as the grandparent for its
file writes (sibling add) but the runtime-authority branch passes
`activePath.phaseId` (the selected child) as `--parent`. Result: `progress.json`
is projected at depth N+1 while `goals.md`, `handoff-in.md`, `scope.json` land at
depth N. Observed creating `codex-harness-comparative-analysis`: runtime put it
under `agui-a2ui-selection-architecture`, files went under the top-level phase.

Fix applied: moved the three script-written files to the runtime's directory and
rewrote `scope.json.allowedWritePaths`. The runtime is authoritative for position.
Root cause is in `~/.claude/skills/kbd-new-child/kbd-new-child.sh` lines 60–66
versus 145–150; not fixed here because it lives outside this repo.

## 2026-09-02 — A filtered registry is not a frozen execution binding

`McpRegistry::filtered` copies eligibility maps but retains shared service slots.
`upsert_server` replaces the slot's transport/config, and reconnect installs a
new client in that same slot. A child using only a filtered view can therefore
execute through a binding different from the one its parent granted. The new
`freeze_bindings` path retains exact transport Arcs, checks selected descriptors
on those transports, and refuses calls after replacement/revocation/closure.
It never reconnects. Child skill activation recognizes this view and checks
inherited dependencies instead of starting source-declared commands/URLs/auth.
Filtering a frozen view preserves its bindings and narrows its close token;
closing a borrowed view does not shut down the parent's transport. This code
has not been built or behavior-tested yet; root capture is still unwired.

The vendored liter-llm client at commit
`c5c6caac617eb931cd5009146a70831422ec236c` also resolves environment credentials
in `DefaultClient::new` (`vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs`).
Do not rebuild a child's client from a copied LlmConfig with missing keys.
`LlmDriver::with_bound_model` now lets supported drivers reuse their captured
client for another model within the same provider; it is not a credential
lookup or an authorization grant. Unsupported host drivers must keep the
original model or supply an explicitly bound implementation, never silently
rebuild against environment/global settings. Manager integration remains open.

## 2026-09-02 — Compile-only checkpoint after root binding capture

The first `cargo check --locked --no-default-features --features server-full`
refused the previously hand-edited lockfile before compilation. Its BackON
entry contained only fastrand, while the new direct dependency enabled timer
features. The cached BackON 1.6.0 manifest and official documentation agree on
that feature wiring: https://docs.rs/backon/1.6.0/backon/ . UAR's actual call
site uses Tokio retry futures. Select `std` and `tokio-sleep` explicitly, without
default browser/blocking sleepers, and include the already-resolved Tokio in
BackON's lock dependencies. No package version or checksum changed.

The subsequent compile found three accumulated errors: an unconditional
reference to the feature-gated in-memory provider, a moved chat input still
needed by capture, and LowerHex formatting on the SHA digest array. Memory
fallback now exists only with its feature; otherwise actor admission requires
configured persistence. Clone the shared input and encode digest bytes using
the existing prompt-hash convention. Public runtime types use redacted Debug
summaries; the legacy vector helper is test-only. The final identical Tier 0
command passed with zero warnings in 30.92s. No tests ran. This supersedes the
earlier no-compilation checkpoint, not any outstanding behavioral acceptance.

Root model and skill captures now have production callers in manager.rs.
Do not overstate this: complete inherited resource bundles, MCP root capture,
child model-policy selection, and ThreadExecutionHost integration remain open.

## 2026-09-02 — Root cost accounting must happen before another model call

End-of-run accounting could only report Exceeded after the tool loop finished;
graph calls bypassed it. Captured drivers now admit against shared scope totals
before each request and atomically apply each priced cumulative Usage update to
run/session/agent/global scopes. A synchronous, short ledger lock avoids losing
part of a received update across scopes when a stream future is cancelled.
Repeated cumulative events replace the same request estimate; later cache counts
can lower it. Run completion reads status instead of charging the estimate again.

Source inventory also corrects an earlier prerequisite: RunManager's only
classifier constructor calls create_classifier, which uses Hybrid for Llm.
create_classifier_with_resources has no production caller under src/. The
standalone LlmClassifier's fresh-client path is therefore not currently in the
shared manager execution path. Do not add its speculative adapter as a
prerequisite; revisit only if that resource-aware factory is actually wired.

## 2026-09-03 — A runner label and a code-shaped argument are not isolation

src/sandbox/wasmtime_runner.rs uses tokio::process::Command and ordinary host
filesystem calls; it does not execute through a Wasmtime sandbox. Its runner_type
and networking capability therefore cannot prove physical isolation. The governed
orchestrator now requires SandboxRunner::enforces_isolation (default false);
RemoteRunner relies on the explicitly configured remote service contract, not
attestation. The legacy local runner itself is unchanged and remains unsuitable
for required isolation.

The same dispatch path previously ignored artifact execution_mode, treated
Sandboxed as Auto, and executed native/MCP directly when a runner or guessed
code field was missing. Wiring actual mode and requiring explicit native sandbox
adapters closes those routes. Never infer code/language from an arbitrary tool
name or code/command/script field. Preserve env/cwd/timeout and configured shell
semantics when adapting terminal execution.

Three T0 passes zero warnings, not runtime acceptance. Remote create/execute/
destroy still lacks joined cancellation ownership and error reconciliation. The
new server caller makes completing that lifecycle necessary before acceptance;
an ignored destroy error or dropped stream cannot count as completed cleanup.

## 2026-09-03 — A consumed join handle must be recorded before another await

In the new owned sandbox worker, JoinError handling awaited the diagnostic
receipt lock before clearing the consumed handle. Cancelling that waiter could
leave a completed JoinHandle to be polled again. Save consumption and a
conservative failed outcome synchronously, then inspect the receipt; a later
waiter can finish classification without re-polling the handle. T0 passed in
24.60s with zero warnings; race behavior remains phase-end verification.

The preceding inline-sandbox lifecycle gap is now addressed by a retained
supervisor with actual orchestrator/manager/server callers. Unknown remote create
or destroy outcomes are still unknown, not cleaned up. Retain their backend and
receipt; do not retry mutations without an idempotency/reconciliation contract.
SandboxBinding now retains actual config and opaque host environment grants.
SandboxConfig::volumes is only a string map; no code here defines read-only mount
semantics. Reject unsupported mounts rather than inventing that wire convention.
The default profile exposes no host mounts/environment and disables networking;
this says nothing about direct native tools outside the sandbox. Final integrated
T0 passed zero warnings20.31s; concrete child-host attachment is still missing.

## 2026-09-03 — Native delegation requires authority, not a ReadOnly label

SessionSearchTool loaded ANONYMOUS_SESSION_OWNER even inside a verified actor
turn. It now receives the frozen verified owner via NativeExecutionContext,
loads only that owner's session and rejects a foreign returned record or missing
host context. Direct child calls also pass through execute_native's mandatory
implementation policy check. ReadOnly alone cannot authorize filesystem/session
access. Unsupported direct tools remain rejected until their permission ports
exist; this is not a completion claim for those tools.

Child registry filtering omitted search_tools. Parent discovery state could
survive descriptor equivalence even though activation and agent controls were
already filtered out. Exclude all three turn-local handler families; activation,
discovery and agent execution check the exact policy they were constructed for.

ThreadService attachment now derives policy/original artifact/persistence/
cancellation and execution host from CapturedThreadKernel. A shared root-owned
atomic claim prevents separate captures from starting two zero-counter schedulers.
No actor/graph/A2A attachment caller exists yet. Five zero-warning T0 passes do
not establish runtime isolation, cancellation or attachment-race behavior.

## 2026-09-03 — Attach actor controls before manifests, retain producer ownership

Supersedes the preceding no-actor-caller observation: manager.rs now captures and
attaches ThreadService for each committed actor root. Its five control names
enter normal policy resolution, not a post-resolution allowlist. The executable
native snapshot must precede installing root handlers; otherwise the service
retains its own handlers through kernel resources. Descriptor-only control
factory identity does not grant spawning or exempt sandbox execution requirements.

ActorCollaboration now creates a child of the live source root. The endpoint is
an explicit verified root-user decision, subject to Cedar and selected policy;
child tools still require the root approval path. An idle source cannot silently
become an independent target root. Raw Collaborate mailbox envelopes are refused.

A completion receiver is not ownership of the tokio producer JoinHandle. Actor
roots now retain that handle, await it by reference, record consumption before
another await, and keep failed receipts. The registry retains roots whose child
cleanup is unresolved after mailbox join. Runtime race behavior is unverified;
latest T0 passed zero warnings23.18s. Remaining native permission ports still
reject unsupported delegated execution; they have not been silently completed.

## 2026-09-03 — Record namespaces and checked network destinations must survive dispatch

Supersedes the older unported compiler/memory/web observations. Compiler tool
session IDs were shared across host conversations; contextual access now uses
the verified owner and host conversation namespace. Legacy NativeTool dispatch
must use call_native_with_context, otherwise memory arguments select user_id
without the admitted turn's owner check. All six memory tools now have that port.
By-ID history cannot infer ownership after the live record has been deleted.

Checking a hostname before making a separately resolved HTTP request does not
bind the request to the checked addresses. WebFetchTool now uses its checked
SocketAddr list for resolution, disallows automatic proxies, and reads only up
to its exact byte allowance. Proxy-only configurations are a compatibility risk;
the existing blocking DNS lookup still has no joined cancellation contract.
Three Tier 0 passes were warning-free (47.45s,32.56s,15.11s); runtime tests remain
deferred. File/patch, direct terminal and A2UI ports are still incomplete.

## 2026-09-03 — File metadata is not a read bound

Native file tools checked floor(len/1024) then read an entire file; growth after
stat and sub-KB overflow bypassed the configured limit. Reads now inspect and
bound one open handle. Patch output expansion is checked before allocating the
replacement, and the same handle is written rather than reopening the path.
Writes flush before success; this is not durable fsync or cancellation rollback.
Append size checks do not lock out unrelated writers. Initial pathname traversal
still lacks directory confinement; child file calls remain denied. Adding the
already-transitive cap-std4.0.2 directly awaits its operator-owned versions.toml
pin. Tier0 passed12.46s with zero warnings; runtime tests stay at phase end.

## 2026-09-03 — Timeout cannot own a terminal process

Direct terminal output() was nested in a timeout; dropping that future discarded
the process handle. Managed calls now use a run-owned TerminalSupervisor with
exact Child handles, borrowed joins and persistent failed-cleanup receipts.
kill_on_drop is fallback protection, not evidence that a process was reaped.
Workers drain stdout/stderr concurrently with bounded head/tail capture.
RunManager and server shutdown have actual cleanup callers. T0 passed twice
zero warnings39.43s,19.92s; no runtime tests yet. A shell's descendants are not
owned by its Child handle. Direct delegated terminal execution remains denied;
standalone raw tools still lack managed joining and bounded raw capture.

## 2026-09-03 — OpenSpec ordinals are not this phase's canonical KBD IDs

kbd-apply list emits ordinal18 with a title beginning4.2. This migrated phase
already registers semantic task ID4.2. Passing18 to begin-task creates another
canonical record rather than starting4.2. A shortened title also fails the
canonical guard's exact-subject lookup. Inspect prometheus kbd status --json
and use the existing semantic ID and exact stored title. This turn's accidental
record18 was cancelled through the typed API, preserving history; the ledger
still counts it (6/26), while OpenSpec remains5/25. Never claim a cancelled
duplicate as another completed implementation task or edit derived projections.

## 2026-09-03 — Latest thread state is not a delegated invocation receipt

A child may finish and start a queued follow-up before its graph parent reads
the watch. Retain the first terminal result separately; do not return the new
turn's result. Likewise, a later unresolved persistence write must not invalidate
an already-committed first receipt. The source critic caught the latter check;
the fix compiles, but race verification remains phase-end integration work.

## 2026-09-03 — Graph early returns bypass shared terminal bookkeeping

The graph producer emitted terminal events and returned before the ordinary
tool loop's RunStatus update. Successful, errored and cancelled graph runs could
therefore stay Running in the active-run store. Each graph exit now sets its
status after cleanup; failed cleanup yields Error, including after cancellation.
Likewise, logging graph shutdown failure is not propagation: the server now
retains the error and returns it, preventing graceful-success reporting.
Source critic re-review found no remaining issue in these two paths. Tier0
passed zero warnings21.23s/17.71s; runtime behavior awaits phase-end tests.

## 2026-09-03 — Failed A2A execution does not prove remote cleanup

An actor can commit a Failed result while its child cleanup remains unconfirmed.
A2A clients must not equate every terminal-looking task with confirmed cleanup.
The shared Task::cleanup_unconfirmed predicate now gates both cancellation
helpers and the retained client driver. The inbound service publishes that flag
before awaiting stop, not just on Err: dropping a stop waiter is also a pending
settlement. Confirmed retry clears only cleanup uncertainty, not historical
execution failure. Exact ActorSession handles avoid spawn-then-name-lookup races;
the registry must retain the whole session, including pending persistence, after
mailbox join. Source review cleared these fixes; runtime tests remain phase-end.

The new A2ATaskExecution stores in-flight mutation futures on the object. A
dropped borrowed waiter can resume them; dropping the whole object is NOT async
cleanup. Its future host caller must retain it until settlement. That graph/
thread integration is still absent; compiling the object is not task4.3 complete.

## 2026-09-03 — Compiler artifacts cannot come from truncated tool history

Native format_result truncates output before chat history and ToolEnd projection.
A signed compiler descriptor cannot be reconstructed reliably from that text.
The actual native host boundary now captures structured compiler results before
formatting, in an owner/run-bound collector closed before the exact actor reply.
Successful tool output survives a later model failure/cancellation without
reclassifying the run as successful. No artifact comes from assistant prose.

CleanupUnconfirmed is not only a child-thread outcome: manager finalizers can
also report sandbox_cleanup_unconfirmed and terminal_cleanup_unconfirmed. A2A
now marks all three, and ActorRootBinding retains those exact resource scopes
for subsequent stop/cleanup. The gRPC Task has no metadata field; uncertain
cleanup remains nonterminal working, not a misleading terminal Failed receipt.
Final source review cleared these paths; runtime checks are still phase-end work.

## 2026-09-03 — Normalize cache tokens before shared budget accounting

Anthropic input_tokens excludes cache reads and cache creation, while UAR's
ModelCost expects cache counts to be portions of an inclusive prompt total.
Passing that field directly undercounts both tokens and spend. StreamState now
sums all three input categories with saturating arithmetic at the provider
boundary. The root budget consumes cache_creation_tokens at catalog cache_write
pricing without counting those tokens twice. Existing compute/estimate_cost
signatures retain their no-cache-write behavior. Pricing remains an estimate,
not an invoice; cache-duration pricing and unreported usage remain limitations.
Source: https://platform.claude.com/docs/en/build-with-claude/prompt-caching
Tier0 passed twice without warnings; runtime regression tests remain phase-end.

## 2026-09-03 — A retained graph-tool future must also survive panic

Keeping a request future in a mutex slot protects it from a dropped node waiter,
but an unwind leaves that slot populated too. Re-polling the already-panicked
future from shutdown can panic outside the producer finalizer's catch and skip
other resource cleanup. GraphToolHost now catches unwinds inside the retained
future and turns them into terminal errors before its slot is cleared.

Shutdown cancels its host token before draining. A cancellation recheck after
ToolStart's event-sink await prevents a never-dispatched request from being
started by that drain. Already-dispatched work is awaited without replay; local
settlement is not proof that a remote effect was rolled back. Source review
accepted the revised path. Runtime cancellation/panic coverage remains phase-end.

## 2026-09-03 — Drain graph work before taking its activation lock

The retained graph model stream can own ActivationContext's mutex while a skill
preflight awaits. A cancelled node waiter releases the host slot mutex, not that
activation guard. Taking the activation lock before polling the retained stream
deadlocks cleanup. Drain the host first, then record activation outcomes.

Panic protection must include transcript finalization and save_session, not only
provider consumption. Persist-failure receipts remain sticky through normal and
cancelled completion and repeated shutdown; a terminal future alone does not prove
history persistence. Child graph hosts also need producer-finalizer ownership when
there is no ActorRootBinding. All are now source-reviewed; runtime races untested.

Graph provider EOF is not success without a normalized terminal event. Node system
prompts are request-local overlays, not shared graph dialogue. Captured children
use frozen MCP registries without preflight objects, so a remote legacy-dispatch
restriction must check inherited thread policy as well as captured preflight.

## 2026-09-03 — A server-name health gauge must aggregate owner bindings

Projected MCP caches isolate by owner/config/auth/environment, but the compatibility
health gauge is intentionally labeled only by server name. Publishing one exact
binding's boolean directly lets a newly observed dormant owner overwrite another
owner's Ready state. Track readiness by lifecycle binding id and publish any-ready.

Compute the aggregate and call the gauge recorder under the same short synchronous
lock. Publishing after lock release races: an older writer can overwrite the newer
aggregate. Unregister on final lifecycle-state drop so dead bindings do not leave a
server permanently healthy. Exact normalized lifecycle events keep their binding id
and sequence; the aggregate applies only to the compatibility gauge.

## 2026-09-03 — Tool success does not prove a valid A2UI projection envelope

An `a2ui_render` ToolResult crosses a host boundary even when `success` is true.
Missing or non-array `a2uiMessages` must emit a protocol error rather than disappear.
Surface IDs are opaque data, not trusted JSON Pointer fragments: reject blank IDs and
encode `~` as `~0` and `/` as `~1` before constructing state paths. Keep projection
in one helper shared by ordinary and graph loops so replay, events and ToolEnd order
cannot drift.

## 2026-09-04 — Preopening a lexical directory name is not enough

A configured path such as `/tmp/..` can look narrower than the directory handle it
opens. Canonicalizing before open also leaves a swap window. For delegated file
authority, open the configured directory first, identity-match its canonical path
back to that exact handle, and compare the handle directly with the filesystem-root
handle. Retain the handle; never reopen the pathname for a child operation.

## 2026-09-04 — MCP transport arguments and HTTP extensions are not authority

An authenticated streamable-HTTP MCP call can still lose identity if a tool
handler reads `user_id` from Parameters. rmcp 3.1.2 preserves Axum request parts
in its tool-call context; recover UserContext from those request extensions and
derive ActorOwner there. Retain the full owner, including tenant, in host-only run
state for later authorization. Comparing only the subject reopens cross-tenant
status access when two issuers reuse a subject.

## 2026-09-04 — Projected MCP revocation and error text need explicit wiring

Mutating the legacy registry does not touch owner-keyed projected bindings.
Every administrative replace, disable, or delete must invalidate all cache keys
for that server and begin transport shutdown. Also never include an expanded
remote URL in an error: URL placeholders may contain credentials and registry
startup logs connection errors.

## 2026-09-04 — Captured environment is not a child-process environment grant

The host needs a complete environment snapshot to resolve declared values and
key bindings, but passing that map wholesale to a skill-contributed stdio server
exposes unrelated database, JWT, provider, and peer credentials. Launch with
`env_clear`, copy only minimal process variables, and add only keys explicitly
declared by that server. Operators must declare every application credential a
server needs.

## 2026-09-04 — MCP service slots need producer accounting independent of registry views

`McpRegistry::merge` and `filtered` can share `ClientServiceState` slots while
holding different registry maps or admission state. Registry-level shutdown
accounting alone is therefore insufficient: a producer admitted through one
view can publish or retire a service after another view's final slot check.
Count replacement producers on the shared slot itself, reject publication once
that slot enters shutdown, and retain rejected/replaced services until an
awaited reap. Likewise, a synchronous removal must transfer the slot to a
shutdown-owned queue before releasing the service-map lock; cancel-without-owning
the eventual join is not a shutdown guarantee.
## 2026-09-04 — KBD/OpenSpec task IDs diverge after semantic import

**Observed behavior.** `kbd-apply list` exposes OpenSpec task ordinals in its
first column, while the canonical KBD phase already contains semantic IDs such
as `1.1` and `1.2`. Passing ordinal `1` to `begin-task` registered a duplicate
canonical row; repeated titles then made the bottleneck guard unable to resolve
a unique task subject.

**Working rule.** In a phase whose KBD task inventory already uses semantic
IDs, drive the KBD transition with that semantic ID and use the OpenSpec ordinal
only for the backend checkbox. Inspect canonical task IDs before the first
phase-end test. A completed duplicate is append-only event history and cannot
be cancelled retroactively, so status must disclose the projection discrepancy.
## 2026-09-04 — Description trimming alone did not preserve skill discovery

**Observed failure.** The 2,000-skill catalog test retained only 1,285 IDs under
the 10,000-token cap even after every description was trimmed away.

**Root cause.** The minimum catalog form still rendered title/source separators
for every row. Their aggregate token cost forced entry omission, so the
description-first policy did not actually preserve the complete eligible set.

**Fix.** After fair description trimming, render an identity-only tier before
omitting entries. If even all IDs cannot fit, omission remains explicit and
counted. The unchanged 2,000-skill integration test now passes with all IDs.

## 2026-09-04 — Retrying a terminated Cargo wrapper left duplicate compilers

The phase-end sidecar build returned exit 1 without a compiler diagnostic.
A subsequent single-job retry did not establish a single writer: process
inspection found two orphaned rustc processes (PPID 1) and a third compiler
owned by the retry, all targeting surrealdb_core with metadata
11b57ce731d471c5 and extra-filename -3b62ef2261cb5ab9 in the same build directory.
The two confirmed orphan processes were sent SIGTERM; the owned retry remained.
An empty tool result or terminated Cargo wrapper does not prove its compiler
children ended. Inspect exact process ancestry, artifact identity and output
directory before retrying. The earlier memory-pressure diagnosis was unproven;
the observed cause of contention was duplicate surviving compiler processes.

## 2026-09-04 — Scratch launchers still discover user skills and need policies

A fresh working directory and database do not isolate UAR from the standard
user skill directory. The existing integration server helper scanned 1,044
skills from the operator's home and twice exceeded its fixed 30-second
readiness deadline before any provider request. A separate attempt became
ready in roughly 27 seconds, proving the timeout was intermittent.

That attempt received real router text, but the child was correctly denied:
the temporary directory contained no `policies/`, so governance loaded an
empty policy set. The live cancellation runner now copies the repository's
three Cedar files unchanged into its scratch directory. Do not bypass
governance or change the home directory to conceal these setup constraints.
Neither startup failure nor successful router text proves child cancellation.

## 2026-09-04 — Shared BDD startup bound was shorter than real skill discovery

The typed-default phase run passed its default/rollback test, then exited 101
at BDD (8/9 scenarios passed). The multi-turn scenario never issued its request:
the helper panicked after 30 seconds waiting for server readiness. Earlier
isolated launch receipts showed intermittent startup exceeding that same bound
while discovering/reconciling 1,044 standard user skills. The shared helper now
allows 120 seconds for readiness and 180 seconds for the enclosing child process;
the existing health probe and request assertions are unchanged. Do not report
this initial run as a product pass; the complete new-default rerun remains the
verification gate. Independent artifact review accepted the bounded adjustment.

## 2026-09-04 — Phase audit supersedes IDs-only catalog and unconditional completion claims

The earlier IDs-only catalog fix preserved IDs but dropped nonempty titles and
suggestion markers. Independent audit found that the 2,000-entry fixture used
empty titles, so its passing result did not prove the written catalog requirement.
The compact tier now keeps titles and suggestions; extreme pressure still uses
explicit counted omission. This supersedes the earlier recommendation to render
an identity-only tier, not the historical test receipt.

The same audit found unused production concurrency (an always-present approval
gate disabled it), retry stopping at provider metadata, primary chat replay
starting another run, and never-dispatched remote leases surviving admission or
cancellation failure. The green phase suite lacked those host-path regressions.
New semantic correction tasks are registered against the four original changes.
Direct Complete-to-InProgress change transition is rejected, but starting a new
task correctly re-derives the change as in progress without rewriting history.
Canonical implementation is therefore 6/10 for the active child and 107/120
overall at revision 2290, not an unqualified 10/10 acceptance.

## 2026-09-04 — Real default-root remote tests exposed routing-mode inheritance

All three new remote host-path tests initially failed before admission with
`unsupported or malformed thread policy section: uar.run_policy.chat_mode`.
The real default-agent root resolves UAR mode; for_remote_child copied that mode
into its named-agent contract, while narrow correctly rejects non-Agent child
mode. The fix belongs in host contract construction: concrete_scope_for selects
Agent mode and preserves the inherited resource/approval/budget ceilings. Do
not mask this by changing the regression fixture to a non-default named root.
The correction's runtime rerun is pending; this records the observed cause, not
a passing result.

## 2026-09-04 — Encoded principals still collide with raw legacy subjects

A collision-safe tenant/subject encoding is not disjoint from arbitrary legacy
subject strings when both use the same table or cache namespace. A subject can
literally equal another principal's encoding. Presentation policy source review
exposed this defect. Verified conversation policies now use separate persistence
tables/maps and a nonnumeric cache prefix; legacy cache keys start with a numeric
length. Legacy fallback cannot grant Presentation intent, and reset markers
suppress fallback. Compilation passed; phase-end regression evidence is pending.

## 2026-09-04 — Omitted policy fields need atomic preservation

Reading saved Presentation intent and later replacing the policy can overwrite
an interleaved restriction. A fresh frontend GET is also not a write baseline:
when Presentation is not dirty, send null/omit it instead of echoing the read
selection. Verified conversation writes now compare their stored policy baseline;
the global field endpoint compares complete raw JSON; agent merge patches use
conditional writes. Global admission bypasses cached values so another host's
restriction and database outages are observed. Backend query execution and
interleaved regressions still need phase-end evidence.

Assignment-load errors must not write the main session save status: a late GET
failure can unlock a form during POST and allow edits that its completion then
discards. The assignment now has separate guarded error state. Confirmed POST
and later derived reads are separate outcomes; uncertain POSTs require an explicit
reread before another save. Do not mark a confirmed mutation failed because its
follow-up effective-state read failed.

## 2026-09-04 — Agent catalog fallbacks are not saved policy authority

The legacy agent list converts a storage failure into built-in defaults. It is
unsuitable for an assignment editor: empty built-in extensions can masquerade as
saved inheritance. The strict persisted-agent GET returns404/503 instead. HTTP
chat also previously resolved built-ins before persistence, bypassing saved
Presentation restrictions; it now reads storage first and propagates failures.
The actor path already used fallible persisted-first resolution. Backend source
review and compilation passed; regression execution remains phase-end work.

Standalone assignment admission includes catalog generation as well as owner.
A same-owner re-admission must invalidate a preflight read before a mutation is
dispatched. Discarded reads publish an explicit retry state, not indefinite
verification. Inactive selected IDs survive confirmed non-Selected saves as
draft metadata; only explicit reset/discard clears them.

## 2026-09-05 — Presentation preparation is not model history

The native orchestrator truncates formatted tool output before ToolResult.
Publishing A2UI by reparsing that text rejects valid large templates and does
not establish frozen-content provenance. Preserve exact untruncated output in
a host-owned, call-ID-bound receipt before formatting; consume it once during
publication. Keep only compact preparation status in model history.

Removing tool IDs does not narrow a registry while effective tool mode stays
Auto/All; convert a narrowed nonempty selection to Selected. Typed prompt
assembly also must explicitly retain host presentation.output instructions.

Legacy chat projection reserves __a2ui_input__/__a2ui_display__ as synthetic
tool names. Provider tool announcements precede execution validation; block
those model-controlled names at the host event boundary. Separately, the
current agui.artifact adapter treats all artifacts as A2UI, including plain
JSON policy output. That adapter defect remains open for the selection UI work.

## 2026-09-05 — Artifact declarations and historical contract presence

The ordinary artifact classifier now gives explicit A2UI intent precedence over
generic JSON, preserving real malformed-profile rejection. Titles must survive
both canonical ContentBlock decoding and toChunks projection; saved derived
chunks alone can conceal a canonical title-loss bug. TypeScript/lint passed;
reload and rendering regressions remain phase-end work.

Serde defaults erase whether older delegation policies omitted Presentation
selection. Retain wire presence and historical typed field order for digest
compatibility; apply target-local None to a separate execution copy. Outgoing
legacy serialization is valid only without negotiation or new template authority.
Do not retry negotiated contracts after removing restrictions. Compilation passed;
old-peer digest and live interoperability are not yet verified.

## 2026-09-05 — Provenance requires host evidence and cursor-safe retention

ToolResult.success is not evidence that a renderer executed. Record generation
failure at the native execution boundary or only after consuming a host receipt.
A missing receipt produces a diagnostic, not a generation-failure claim.
NormalizedEvent::Error closes run streams; recoverable Presentation rejection
must be nonterminal so text fallback and final provenance can arrive.

The512-event history ring can evict admission evidence. Keep the latest full
provenance with its original sequence separately; never show it at an older
cursor. Unrelated incomplete A2UI state must not erase independently known
provenance or cause a fabricated synchronized global snapshot. A dedicated
CUSTOM snapshot extension preserves that distinction. These paths compile and
passed source review; phase-end cursor/eviction/ordering tests are still pending.

## 2026-09-05 — Phase test fixtures and stale-callback evidence

Archiving typed-turn-assembly moved parity-report.json, but a Rust include_str
still referenced the active change directory. Preserve the oracle contents and
update the exact path; do not regenerate expected parity just to pass the test.

A UI run-switch test can miss stale writes to the previous run because it now
subscribes to a different graph ID. Assert the prior entity remains idle with
no observation after invoking the old callback. Also retain the existing A2UI
protocol ban on executable markup when testing literal template data.

## 2026-09-05 — Presentation phase browser and fixture findings

A Button onClick callback must not pass its MouseEvent to an action whose
optional argument is a record ID. NewPresentationButton did so; the owner-safe
domain rejected the event as an unknown ID. Explicitly call onOpen() and test
both creation entry points plus existing-row identity.

Base UI Select.Value needs the existing value-to-label items mapping when the
closed trigger must display authored labels rather than wire values. Include
dynamic 'Inherit, with exclusions'; exact-text tests distinguish its two states.

An explicit --env-file does not isolate the UAR executable from cwd .env:
main.rs calls dotenv() first, and legacy LLM_* values override the config file.
A phase browser fixture launched from the repo imported an existing credential
into its temporary database. Launch test executables from a clean temporary cwd
with explicit test-only env and config; verify actual persistence descriptors
and sanitized effective settings before navigating credential-bearing pages or
sending model requests. Never log the credential. The original env file was
unchanged; the exposed key requires operator rotation.

## 2026-09-05 — Run inspector intrinsic width and publication vocabulary

An implicit single-column grid can use a JSON pre block's min-content width:
the390px trace expanded to26361px. Explicit grid-cols-1 plus shrinking inspector
and tab-panel children keeps overflow local to the JSON pane. The existing
Tabs wrapper rendered flex-direction row in this inspector; a local flex-col
keeps the tab strip above its panel without refactoring every shared Tabs user.
Confirm real computed width, not just document scrollWidth, because an ancestor
can hide internal overflow. The Presentation metric deliberately excludes
policy-summary artifacts; label it generated UI surface publication so visible
diagnostic UI does not contradict an empty renderer-publication result.
