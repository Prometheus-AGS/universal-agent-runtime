# Goals

Phase: **uar-security-deps-and-hygiene**

Seeded from `uar-spec-v2-and-polish`'s reflection.md — specifically its
2026-07-04 addendum, which rescoped the original "Next Phase Focus"
after a Dependabot alert backlog was discovered post-reflection (96 open
alerts, 5 critical/17 high, ~4 months accumulated, never investigated
because it wasn't in that phase's declared scope).

## G1 — Security dependency triage & upgrade (P0, primary)

- **Triage first.** Confirm actual exploitability/reachability for the 5
  critical + 17 high Dependabot alerts before upgrading anything blindly
  — don't assume every CVE title is exploitable in UAR's specific usage
  pattern. Start with `surrealdb` and `rmcp` (see below); `wasmtime` and
  `failure` are lower priority (rationale below).
- **`surrealdb`**: pinned `=3.0.5` in `Cargo.toml`; crates.io currently
  has `3.2.0`. High-severity alerts include an HTTP RPC session-UUID
  leak (anonymous session hijack) and a privilege-escalation race
  condition via HTTP RPC. `surreal-backend` is UAR's **default**
  feature — this is the persistence layer most deployments actually
  run. Follow the existing upgrade SOP in `docs/DEPENDENCY_MANAGEMENT.md`
  rather than improvising one; re-run the full test suite after
  upgrading and disclose any behavior changes found.
- **`rmcp`**: pinned via git `rev` in `Cargo.toml`, well behind
  upstream's current `HEAD`. High-severity DNS rebinding vulnerability
  in its Streamable HTTP server transport. `rmcp` is the core,
  non-optional MCP SDK — "MCP-first tool execution" is a headline
  architecture pillar. Bump the pin to a commit that includes the fix;
  verify MCP client/server paths still work end-to-end.
- **`wasmtime`/`wasmtime-wasi`**: 2 critical sandbox-escape bugs
  (aarch64 Winch backend) + a WASI `path_open(TRUNCATE)` bypass. Lower
  priority than the two above — `wasm-runtime` is opt-in
  (`Cargo.toml`'s `default = ["surreal-backend"]` does not include it),
  so exposure is limited to deployments that explicitly enable it.
  Still worth a version bump if straightforward; otherwise document the
  residual risk explicitly rather than silently carrying it.
- **`failure` crate**: critical type-confusion alert, but this is a
  dev-only transitive dependency of `grcov` (coverage tooling) — never
  shipped in any built artifact. Confirm this via `cargo tree -i
  failure` (already done during triage research: only reachable via
  `[dev-dependencies] -> grcov -> cargo-binutils -> rustc-cfg ->
  failure`) and explicitly disposition as "no action needed, no
  production exposure" rather than silently ignoring it — a disclosed
  non-issue, not an unexamined one.
- **npm-side alerts** (`dompurify`, `jsonwebtoken`, `vite`, `undici`,
  `minimatch`, etc.): mostly medium/low and likely build-tooling
  transitive dependencies (vite/vitest), but `dompurify` (XSS-relevant —
  used for sanitizing rendered chat content somewhere in the dependency
  graph) and `jsonwebtoken` (auth-relevant) are security-sensitive by
  nature even if not found as direct `frontend/package.json` entries.
  Trace their actual reachability and patch what's realistic.
- **Add `.github/dependabot.yml`** for version-update automation so
  this backlog doesn't silently reaccumulate to 96 again with nobody
  noticing.

## G2 — Hygiene & validation (P1, secondary — carried from `uar-spec-v2-and-polish`)

- **Automate the artifact-refiner QA gate**, or make an explicit,
  disclosed decision to drop it from this project's KBD contract. This
  is its 4th+ consecutive phase as unaddressed debt.
- **Fix `tests/uar_integration.rs`**: a `Skill` struct literal is
  missing 8 fields, breaking `cargo check --tests` for that target.
  Pre-existing, unrelated to any change in the prior phase.
- **Fix `tests/bdd.rs`**: a nested `#[path]` attribute resolves
  incorrectly (`tests/live/integration/live/harness.rs`, which doesn't
  exist). Also pre-existing and unrelated.
- **Run `benches/hot_path.rs`** via `cargo bench` (and at minimum
  `cargo check --benches`) — it shipped in the prior phase (CH-20) but
  has never actually been compiled or executed in any session.
- **Fix `write-position-reminder.sh`'s `.stage`/`.status` field
  mismatch** at the source (script or schema) — it silently rendered
  `Stage: unknown` for most of the prior phase until patched by hand.

## Operator-only follow-up (carried, not agent-actionable)

- Activate the eval gate: set `UAR_LLM__API_KEY` secret +
  `vars.UAR_EVAL_MODEL`, run `eval-nightly` via `workflow_dispatch
  update_baseline=true`, commit `evals/results/starter.baseline.json`.
  Carried across multiple phases now.

## Success criteria

- Every G1 item is either fixed-and-verified (full test suite green
  post-upgrade) or explicitly dispositioned with rationale (e.g.
  `failure` crate, `wasmtime` residual-risk decision) — not silently
  dropped.
- No new Dependabot alert count regression from any version bump
  performed (verify via `gh api repos/.../dependabot/alerts` before and
  after).
- G2 items addressed or explicitly re-carried with a reason, matching
  this project's established carry-over discipline.

---

## Instructions

Review and refine the goals above before running `/kbd-assess`. When
ready:

```
/kbd-assess uar-security-deps-and-hygiene
```
