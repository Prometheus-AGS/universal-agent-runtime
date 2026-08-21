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
