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
