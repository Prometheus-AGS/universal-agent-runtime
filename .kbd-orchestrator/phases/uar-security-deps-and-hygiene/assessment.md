# Assessment — uar-security-deps-and-hygiene

**Date:** 2026-07-04
**Method:** direct inspection (`gh api` dependabot alerts, `cargo tree`,
`Cargo.toml` feature flags, `git log`/`git ls-remote`, `cargo check`,
`pnpm-lock.yaml` grep) — not assumption, per the standing lesson from
prior phases ("verify against direct evidence before assuming status").

## G1 — Security dependency triage & upgrade

### `surrealdb` — PARTIAL (pin confirmed stale, upgrade not started)

Pinned `=3.0.5` in `Cargo.toml:232`; crates.io currently offers `3.2.0`
(2 minor versions ahead). `surreal-backend` is the crate's `default`
feature (`Cargo.toml:69` — "SurrealDB embedded only (SurrealKV), no
Postgres/SQLite"), so this is the persistence layer nearly every
deployment uses. Confirmed via `gh api
repos/.../dependabot/alerts`: multiple **high**-severity open alerts
against this dependency, including an HTTP RPC session-UUID leak
(anonymous session hijack) and a privilege-escalation race condition via
HTTP RPC — plus 20+ medium-severity entries (DoS vectors, permission
bypasses). Whether these are *reachable* depends on deployment mode:
UAR's default is embedded SurrealKV (in-process, no network RPC
surface at all — these specific HTTP-RPC CVEs likely don't apply);
the Helm chart's `surrealdb-statefulset.yaml`/`surrealdb-service.yaml`
path runs SurrealDB as a **networked server** with HTTP/WS RPC exposed
(`ws://surrealdb:8000` per `values.yaml`), and *that* path is
plausibly reachable within the cluster network even with the existing
`NetworkPolicy` restricting access to the UAR pod only. Not yet
determined: whether `docs/DEPENDENCY_MANAGEMENT.md`'s existing upgrade
SOP has been exercised against 3.2.0 specifically, or whether 3.1/3.2
introduce breaking schema/query changes against the 12 SurrealDB
migrations already in this repo.

### `rmcp` — PARTIAL (pin confirmed stale, upgrade not started)

Pinned via `rev = "085470025f690050e8776ffa939e7ba71d3abc01"`
(`Cargo.toml:150`); `git ls-remote` against
`modelcontextprotocol/rust-sdk` shows current `HEAD` at
`bdf0c32e8c1ea1847ab9c581c0ee0d4984d6b556` — a materially different
commit. Confirmed one **high**-severity open alert: DNS rebinding in
rmcp's Streamable HTTP server transport. `rmcp` is the core MCP SDK
(non-optional dependency) — "MCP-first tool execution" is a stated
architecture pillar. Not yet determined: which commit between the pin
and `HEAD` actually introduced the fix, or whether the intervening
history contains breaking API changes for this repo's MCP client/server
integration points.

### `wasmtime` / `wasmtime-wasi` — NOT STARTED, correctly lower priority

Confirmed via `Cargo.toml:95`: `wasm-runtime = ["dep:wasmtime",
"dep:wasmtime-wasi"]` is **not** part of `default = ["surreal-backend"]`
— opt-in only. 2 open **critical** alerts (aarch64 Winch-backend
sandbox escape) + 1 high (`wasmtime-wasi` `path_open(TRUNCATE)` bypass
of `FilePerms::WRITE`). Exposure is real only for deployments that
explicitly enable `wasm-runtime` — unknown from this assessment alone
whether any current deployment does. Needs an explicit disposition
(bump version, or document residual risk) rather than silent carry.

### `failure` crate — DONE (disposition confirmed, no action needed)

`cargo tree -i failure` confirms the only path is `[dev-dependencies]
universal-agent-runtime → grcov → cargo-binutils → rustc-cfg → failure`
— a coverage-tooling transitive dependency, never compiled into any
shipped artifact. The **critical** type-confusion alert against it
carries no real production exposure. This is a "disclosed non-issue,"
not unexamined — no further action needed beyond recording this
finding.

### npm-side alerts (`dompurify`, `jsonwebtoken`, `vite`, `undici`, `minimatch`, etc.) — STUB (traced partially, not resolved)

`dompurify@3.4.7` confirmed present in the root `pnpm-lock.yaml`
(matches the version in the open medium/low alerts: `ALLOWED_ATTR`
pollution, Trusted-Types-policy survival, `SAFE_FOR_TEMPLATES` bypass).
It is not a direct dependency of `frontend/package.json` or the root
`package.json` — only `@types/dompurify` appears as a direct
devDependency, so the runtime `dompurify` package is pulled in
transitively (likely by a Markdown-rendering or HTML-sanitization
library used somewhere in chat-content rendering, given
`chat-messages`/code-block/Mermaid components exist per `CLAUDE.md`,
but the actual importer was **not** traced this pass). `jsonwebtoken`
was **not found** in either `pnpm-lock.yaml` at the paths checked
(root, `frontend/`) despite appearing in the Dependabot alert list —
it may live in a different lockfile (a git submodule such as
`prometheus-entity-management` or the skill-pack, neither of which was
checked). This needs a dedicated trace pass, not a guess.

### `.github/dependabot.yml` — MISSING (confirmed absent)

`ls .github/dependabot.yml` — file does not exist. Security *alerts*
are on (GitHub's passive scanning), but there is no automated
version-update PR pipeline. This is why 96 alerts accumulated silently
over ~4 months with nobody surfacing them as reviewable diffs.

## G2 — Hygiene & validation (carried from `uar-spec-v2-and-polish`)

### Artifact-refiner QA gate — MISSING (confirmed, not just assumed)

`ToolSearch` for artifact-refiner tooling in this session returned no
matches — there is currently no invokable artifact-refiner MCP tool
available at all in this environment, not just "not wired into this
project's KBD flow." This is a 4th+ consecutive phase where this gate
has gone unaddressed; G2 asks for either real automation or an
explicit, disclosed decision to drop it from this project's KBD
contract, since "carry forward again" without a decision is itself a
form of scope drift.

### `tests/uar_integration.rs` — CONFIRMED STILL BROKEN

Re-ran `cargo check --test uar_integration` this session:
`error[E0063]: missing fields authors, compatibility, language and 5
other fields in initializer of Skill` at `tests/uar_integration.rs:430`.
Unchanged since `uar-spec-v2-and-polish`'s discovery of it — a
straightforward mechanical fix (add the 8 missing fields to the
`Skill` literal), not yet attempted.

### `tests/bdd.rs` — CONFIRMED STILL BROKEN

Re-ran `cargo check --test bdd` this session: the nested `#[path =
"integration/live/harness.rs"]` inside `mod live { ... }` resolves
relative to `tests/live/` (since the outer `mod live` has no `#[path]`
of its own), producing a nonexistent path
(`tests/live/integration/live/harness.rs`). Unchanged since discovery.
Fix: either give `mod live` its own explicit `#[path = "."]`-relative
handling, or flatten the nested-module `#[path]` attributes to be
directly relative to `tests/` instead of `tests/live/`.

### `benches/hot_path.rs` — CONFIRMED NEVER RUN

No evidence in this session (or referenced in any prior phase's
verification notes) that `cargo bench --bench hot_path` or even `cargo
check --benches` has ever been executed. The benchmark code has only
been reviewed by inspection since it was written in
`uar-spec-v2-and-polish`'s CH-20.

### `write-position-reminder.sh` `.stage`/`.status` mismatch — CONFIRMED, PARTIALLY MITIGATED

The script (`shared/scripts/write-position-reminder.sh`) reads
`jq -r '.stage // "unknown"'` from `current-waypoint.json`. This
project's waypoint schema historically only populated `.status`, not
`.stage` — confirmed by re-reading the script's source this session.
A `stage` field was added by hand during `uar-spec-v2-and-polish` as a
workaround (present in `current-waypoint.json` today), but the
underlying script/schema mismatch is not fixed at the source — a future
waypoint write that omits `.stage` (e.g., from a tool that doesn't know
about this project-specific workaround) would silently regress to
`Stage: unknown` again.

## Cross-Tool Progress

NONE — no Roo/Cursor/Codex/Antigravity/OpenCode activity recorded in
`progress.json` for the prior phase or referenced anywhere for this one.
Single-tool (`claude-code`) continuity from `uar-spec-v2-and-polish`
straight into this phase.

## Build Health

- `cargo check --lib`: **PASS** (clean, 1.37s incremental).
- `cargo check --test uar_integration`: **FAIL** (pre-existing,
  unrelated to any tracked change — see above).
- `cargo check --test bdd`: **FAIL** (pre-existing, unrelated — see
  above).
- Known violations: the two failures above; no others surfaced by this
  pass.
- Test coverage for this phase's own scope: N/A — nothing implemented
  yet (assess-only pass).

## Constraint Compliance

- `AGENTS.md` has no literal "Never Do" heading (confirmed by `grep`);
  the applicable rule set is the 40-rule Prometheus Base Rules Set
  mirrored in both `AGENTS.md` and `CLAUDE.md`. No violations to report
  at this stage — no code has been written yet.
- `.kbd-orchestrator/constraints.md` does not exist (confirmed) — no
  separate machine-checkable constraint file beyond the rule set above.
- Rule 22/23 relevance: this phase's own G1 is, in effect, "verify
  dependency versions against official sources before assuming a pin is
  still fine" — directly the rule this project's own D-D decision
  should have been checked against during `uar-spec-v2-and-polish` and
  wasn't.

## Spec Gap Summary

- No canonical spec file enumerates a target dependency-freshness
  policy or a security-alert SLA for this project. `DEPENDENCY_MANAGEMENT.md`
  documents *why* pins exist and *how* to upgrade them, but not *when*
  an upgrade becomes mandatory (e.g., "upgrade within N days of a
  high/critical alert"). Worth considering as an artifact of this
  phase, not assumed as already covered.

## Goal Progress

| Goal | Status | Reason |
|---|---|---|
| G1 Security dependency triage & upgrade | **NOT MET** | Triage done for surrealdb/rmcp/wasmtime/failure (this assessment); npm-side triage incomplete (jsonwebtoken not located); no upgrades performed yet; `dependabot.yml` not yet added. |
| G2 Hygiene & validation | **NOT MET** | Both pre-existing test failures reconfirmed broken; artifact-refiner gate confirmed unavailable (not just unused); `benches/hot_path.rs` confirmed never run; waypoint script mismatch confirmed only partially mitigated. |

## Sycophancy self-check

- S-03: at least one concern surfaced per section above (surrealdb
  deployment-mode ambiguity, rmcp commit-range unknowns, jsonwebtoken
  untraced, artifact-refiner gate literally unavailable in this
  environment) — this assessment is not friction-free.
- S-02: the "embedded SurrealKV likely isn't exposed to the HTTP-RPC
  CVEs" claim is explicitly qualified as deployment-mode-dependent, not
  asserted as blanket safety.

ASSESSMENT COMPLETE
