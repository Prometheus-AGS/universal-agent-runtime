# Analysis — uar-hybrid-app-architecture

> Mode: stack specified (Tauri + React 19 desktop/web, Flutter + Rust FFI mobile,
> per the operator's hybrid-mobile-architecture skill / TJ-ARCH-MOB-001).
> Research run 2026-07-15; budget used: T1 2 queries, T3 3, T4 2 — well under caps.
> Input: assessment.md (7 goals) + seed-analysis.md.

## Landscape and candidate evaluation

### G3 desktop data layer — pglite-oxide (cand-001) — ADOPT, gated
Operator-directed and skill-matrix-aligned. Verified today (Tier 1/3):
MIT, 0.5.1 (2026-06-04), 4,192 crates.io downloads, repo pushed **today**
(2026-07-15) — small (90 stars) but alive and moving.

**New finding the skill doc does not carry:** release AOT assets are
`aarch64-apple-darwin`, `aarch64/x86_64-unknown-linux-gnu`,
`x86_64-pc-windows-msvc`, plus `portable-wasix` — there is **no
x86_64-apple-darwin AOT asset**, and this project's primary dev machine is an
Intel Mac. The portable WASIX build presumably runs there (JIT, slower start),
but that is unverified. The Intel-mac portability spike (change 2) still
determines whether pglite-oxide is wired in on that platform.

**REVISED 2026-07-15 (operator decision), supersedes the "second engine"
concern below:** rather than falling back to a *third* distinct local engine
("webview PGlite on a stable origin") when pglite-oxide isn't available,
desktop uses **embedded SurrealDB (surrealkv/kv-rocksdb)** — the same engine
UAR's server and the mobile target (G4, cand-009) already use — as the
universal baseline backend behind the repository traits. pglite-oxide layers
in as an optional, platform-gated enhancement (PG-wire compatibility, shared
SQL schema with web/cloud) rather than a requirement. This fully resolves the
second-engine concern: SurrealDB was already a UAR dependency everywhere;
nothing new is required for the change to ship a working desktop data layer,
and desktop's data layer now consistently uses the same backend as mobile.
~~Second engine concern (pglite-oxide beside UAR's SurrealDB) stands but is
accepted by operator direction~~ — superseded; SurrealDB is the baseline, not
an addition beside it. pglite-oxide, where adopted, is genuinely additive.

### G3 web data layer — @electric-sql/pglite (cand-002) — ADOPT (status quo)
Already in use at the skill's prescribed configuration (`idb://uar-threads`).
No change; matrix row already compliant.

### G2 sidecar — Tauri externalBin + plugin-shell pattern (cand-005) — ADOPT
First-party Tauri v2 mechanism; `@tauri-apps/plugin-shell` already a frontend
dependency; `bundle.externalBin` already carries two MCP servers, so the
packaging path is proven in-repo. `uar-sidecar` binary exists and is
installed today. Remaining product decision (operator): sidecar lifetime
after app exit — die-with-app vs. keep-serving-OS-consumers.

### G4 mobile FFI seam — flutter_rust_bridge (cand-003) — ADOPT
2.12.0 on crates.io, updated 2026-07-11 (four days ago). The skill's
prescribed bridge; its scaffold scripts (cand-008) assume it.

### G4 mobile vector — sqlite-vec (cand-004) — ADOPT
7,886 stars, Apache-2.0, prebuilt iOS/Android libs per skill doc; last push
2026-05-18 (~2 months — acceptable, watch cadence).

### G4 mobile graph — SurrealDB kv-rocksdb (cand-009) — ADOPT
Per the skill's corrected matrix; UAR already embeds SurrealDB (surrealkv),
which is the on-device persistence that makes "UAR entirely on mobile"
feasible. Evidence tier: skill doc (verified today) — no independent
re-verification burned this round.

### G4 scaffolding — hybrid-mobile-architecture-skill (cand-008) — ADOPT
`scripts/scaffold-hybrid.sh` / `scaffold-flutter.sh` +
`references/arch-standard.md` are the entry point; templates for Flutter
features and Rust core ship in `assets/templates/`.

### G7 TypeScript 7.0 (cand-006) — ADAPT (hybrid), not full adopt yet
Verified: TS 7.0 GA'd 2026-07-08; the standard `typescript` npm package now
ships the Go compiler. **Blockers found (Tier 4):**
- typescript-eslint supports `>=4.8.4 <6.1.0` — TS 7 not supported; users
  report crashes (typescript-eslint#12518, #12521).
- typedoc depends on TS 6 internals absent from the TS 7 API; rewrite
  pending, feature-freeze planned (TypeStrong/typedoc#3098).
- Microsoft ships `@typescript/typescript6` (cand-007) exactly for this:
  tsc on 7, API-consuming tooling on 6.

**Verdict:** hybrid migration — move build/typecheck (`tsc -b`) to TS 7,
pin typescript-eslint/typedoc to the `@typescript/typescript6` compat
surface; revisit full migration when both tools ship TS 7 support (their
trackers above are the watch signals). Vite/rolldown transpilation does not
consume the TS compiler API (type-stripping path), so the build pipeline
itself is low-risk — moderate confidence, verify in the change.
Sequence AFTER the UI remediation waves to avoid churn collision.

## Build-required (no viable library)
- G1 stable-port fix — internal change to src-tauri (no dependency).
- G5 /impeccable program — internal skill execution + remediation waves.
- G6 supplemental changes 1–6 — internal (already planned in prior phase).
- Governance tool-approval reconciliation — internal investigation.

## Open questions (carried to Spec/Plan)
1. **Sidecar lifetime policy** (operator decision) — die-with-app or persist.
2. **pglite-oxide on Intel mac** — RESOLVED 2026-07-15: spike outcome now
   only gates whether pglite-oxide is additionally wired in as an
   enhancement; embedded SurrealDB is the universal baseline regardless
   (operator decision, see above).
3. **Mobile scope** — G4 is likely a child phase (`/kbd-new-child`) rather
   than in-phase changes; recommend deciding at plan time based on change
   count (if mobile alone exceeds ~6 changes, split).
4. **TS 7 timing** — hybrid now vs. wait for 7.1 + tooling; recommendation
   above is hybrid-after-UI-waves, operator may prefer to defer entirely.

No contested stack choice arose (stack was operator-specified); no
elicitation required.

## Cross-validation against the hybrid-mobile-architecture skill's reference
## implementation (2026-07-15, post-plan)

Reviewed the local, more-current checkout at
`/Users/gqadonis/Projects/references/hybrid-mobile-architecture-skill`
(github.com/Know-Me-Tools/hybrid-mobile-architecture-skill), which now
contains a **verified-building reference scaffold**
(`apps/knowme-poc`, commit `86e7d1d "verify knowme-poc scaffold builds and
runs on desktop (Tauri) and web"`) rather than just documentation — a
materially stronger evidence source than the doc-only research this
analysis originally ran on. Findings below either confirm or refine
earlier verdicts; none invalidate the existing plan.

### Confirmed: liter-llm as the cloud LLM gateway (independent validation)
The reference app's `.kbd-orchestrator/phases/*/plan.md` was revised
same-day (commit `4ed2d08`, 2026-07-15) to replace a bespoke Anthropic SSE
client with **the liter-llm gateway (GQAdonis/liter-llm fork)** — the exact
fork UAR already depends on and patched today for the SSE streaming bug.
This is strong independent cross-validation, not circular (the revision
was written before this analysis reviewed it). No plan change needed —
UAR was already correctly positioned here.

### Confirmed: Tauri sidecar-HTTP pattern is sound for UAR's specific needs,
### and is a deliberate, reasoned divergence from the skill's own reference
### pattern — not an oversight
The reference app's actual desktop shell
(`apps/knowme-poc/desktop/src-tauri/src/lib.rs`) uses **pure Tauri IPC**
(`invoke_handler!` + `#[tauri::command]` functions in `commands.rs`) with
**no embedded or sidecar HTTP server at all** — no axum, no TcpListener,
anywhere in the shared `gen_ui_core` crates. The frontend calls `invoke()`
exclusively; grep for `fetch(` across `desktop/src` returns nothing.

This is the skill's canonical pattern for a **greenfield single-UI app**.
UAR is not that case: it is an existing ~100-endpoint REST/AG-UI/A2UI HTTP
server whose entire React frontend already calls `fetch()` against relative
URLs, and the operator's stated requirement is explicit network-level
exposure ("exposing the port to the operating system container" — i.e.
other local processes, not just the one webview, must reach it). A
same-day web search
([tauri-apps/tauri sidecar docs](https://v2.tauri.app/develop/sidecar/);
community sidecar-HTTP-server examples) confirms spawning a sidecar that
serves HTTP to both the internal webview and external local consumers is
an officially-supported, well-precedented Tauri pattern — just a different
one than this particular reference app happened to need.
**Conclusion: plan changes 10 (desktop-sidecar-conversion) and 1
(desktop-stable-port) remain correct as planned.** Recording this
divergence explicitly so it reads as a reasoned decision, not an
unreviewed gap, if this analysis is revisited later.

### New consideration (not actionable now): config-DB pattern for provider/
### model settings
The reference app's C-103 change stores provider/model configuration in a
schema'd config DB (`providers` / `model_prefs` / `app_settings`,
pglite-oxide on desktop/mobile, PGlite on web) and states explicitly:
"Provider/model selection reads the config DB, never env vars." This is
architecturally the same problem class as the Admin UI findings from
`uar-grade-a-upgrade-2026-07`'s supplemental assessment (provider/model
resolution split across the agent artifact's `policy.provider`, the
`ProviderRegistry`'s in-memory default, and a settings-DB `llm.model` row,
with unclear precedence — the root cause of the Orchestrator/gpt-5.2 bug
fixed earlier today). Change 6 (`admin-agent-provider-first-model-picker`)
and change 11 (`desktop-data-layer-pglite-oxide`) should keep this pattern
in mind as a design reference when the desktop data layer is actually
implemented, but **no scope change to either change now** — UAR's existing
provider/model resolution already works (verified live today) and a
consolidation is a larger refactor than either change's current bounds.
Flagging for whoever implements change 11 to revisit.

### New consideration (out of current scope): on-device/local inference
### lane
The reference app pairs its cloud lane (liter-llm) with a **local native
inference lane** — `mistral.rs` (GQAdonis/mistral.rs fork of
EricLBuehler/mistral.rs, verified via `gh api`: real fork, pushed
2026-06-03) behind a `gen_ui_inference` crate, running a small quantized
model (Qwen2.5-1.5B-Instruct Q4_K_M) on-device via Metal (desktop) or CPU
(mobile) — plus a researched web-only lane (WebLLM/MLC on WebGPU) with
visible degrade to cloud. None of this phase's 7 goals currently call for
on-device model inference; UAR's mobile goal (G4) is scoped to on-device
*UAR persistence* (surrealkv) reaching a *cloud* LLM provider through the
normal liter-llm routing, not local model execution. Flagging as a
candidate for the mobile child phase's own assessment if "sovereign /
offline-capable mobile UAR" becomes an explicit goal later — not adding it
speculatively now (YAGNI).

### Re-confirmed, not re-solved: pglite-oxide Intel-mac gap
Checked the reference project's `decision-log.md` and
`docs/pglite-oxide-tauri-hybrid.md` for any mention of `x86_64-apple-darwin`
or Intel Mac — none found. Their own testing is Apple Silicon + iOS
simulator only (per commit messages). This does not resolve the gate
already recorded above (`pglite-oxide-intel-mac-spike`, change 2); it
independently confirms no one has solved it yet, so the spike remains
necessary rather than skippable.
