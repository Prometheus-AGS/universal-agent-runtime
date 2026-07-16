ASSESSMENT: uar-hybrid-app-architecture
Project: universal-agent-runtime
Date: 2026-07-15
Codebase baseline: 25/25 Grade-A changes merged (prior phase); live server verified working today through the uar-jwt-proxy; Tauri shell exists but predates this phase's architecture goals.
Cross-tool progress: NONE — phase created today; no other tool has recorded work.

Inputs: goals.md (7 goals), seed-analysis.md (live-verified findings from today's
operator-directed investigation — treated as prior context, spot-re-verified below),
openspec/specs/ (67 specs), direct codebase inspection, web verification for TS 7.0.

IMPLEMENTATION STATUS (per goal)

- G1 stable-port fix: MISSING — re-verified: src-tauri/src/lib.rs::resolve_localhost_port
  still binds 127.0.0.1:0 (random port per launch) unless TAURI_LOCALHOST_PORT is set,
  then navigates the webview to that origin. Every desktop launch orphans the per-origin
  browser state: PGlite thread DB (idb://uar-threads), localStorage (theme, onboarding),
  and SW caches. P0 data-integrity defect; smallest goal; no dependencies.

- G2 sidecar conversion: MISSING — the shell embeds the server in-process
  (server::start_server on Tauri's async runtime). uar-sidecar binary exists, is built by
  cargo install today, and is the intended vehicle. bundle.externalBin currently lists only
  two MCP servers. plugin-shell is already a frontend dependency. The /readyz wait loop in
  lib.rs transfers directly. Blocker to note for plan: sidecar lifecycle (kill on app exit
  vs. leave running for other OS consumers) is an unmade product decision the operator must
  call.

- G3 per-target data-layer alignment: PARTIAL —
  * Web: DONE. frontend/src/lib/db.ts uses PGlite 0.5.x at idb://uar-threads; matches the
    skill's matrix row exactly.
  * Desktop: MISSING. pglite-oxide appears nowhere in the workspace (checked root and
    src-tauri Cargo.toml). The open design decision from seed-analysis.md stands: adding
    pglite-oxide introduces a second storage engine beside UAR's SurrealDB — needs an
    explicit adopt/defer decision in Analyze/Spec, not a silent default.
  * Mobile: MISSING (see G4).

- G4 mobile target (on-device UAR): MISSING — no pubspec.yaml, no gen_ui_core, no Flutter
  or FFI scaffolding anywhere in the repo. This is greenfield inside this repo; the
  hybrid-mobile-architecture skill's scaffold scripts are the intended entry point.
  Feasibility pre-verified: UAR's canonical persistence is embedded SurrealDB (surrealkv,
  pure Rust), so on-device UAR does not depend on Postgres-on-mobile. Largest goal; likely
  its own sub-phase or child phase.

- G5 /impeccable UI/UX program: MISSING (not yet run) — structural evidence already
  supports the operator's "brittle" characterization:
  * frontend/src/admin/pages/settings-page.tsx is 3,336 lines — 4.2x the project's own
    800-line hard cap; chat-stream-store.ts is 1,200; models-page.tsx 893. Three more
    files exceed 700.
  * 254 non-test TS/TSX files, ~31.9k lines, but only 29 test files (~11% of files have
    any test) — far from the 80% coverage bar the project rules set.
  * Known defect inventory from today's live testing: sw.js scheme crash, warning-icon
    semantics, flat model picker, unverified Edit Agent save paths, governance UI
    disconnect, unreproduced freeze (PGlite main-thread init suspect).
  The audit/critique run itself is execution work; assessment records the baseline.

- G6 absorb 6 supplemental Admin UI changes: PLANNED/0 of 6 — fully specified in the
  prior phase's plan.md supplemental section and assessment.md; no implementation started.
  Change 4 depends on change 3; changes 5–6 are investigation-first.

- G7 TypeScript 7.0 migration: NOT STARTED; VERIFICATION GATE NOW SATISFIED FOR "EXISTS" —
  web-verified today: TS 7.0 reached GA 2026-07-08 (one week ago); the standard
  `typescript` npm package now ships the Go-native compiler; ~8–12x faster builds; strict
  mode + 6.0 deprecations become hard defaults. CRITICAL CAVEAT: 7.0 ships without a
  stable programmatic API (deferred to 7.1) — anything in our toolchain that consumes the
  TS compiler API (typedoc in the docs pipeline, typescript-eslint's type-aware rules,
  any Vite plugin using the TS API) must be individually compatibility-checked in Analyze
  before committing. Current state: typescript ^5.9.3, `tsc -b` typecheck passes clean
  today. Migration risk is concentrated in tooling, not app code. GA is 7 days old —
  early-adopter risk is real and should be priced into sequencing (after the UI waves,
  not before, per seed-analysis.md's suggested shape).

CROSS-TOOL PROGRESS
NONE — no cross-tool activity recorded for this phase.

SPEC GAP SUMMARY
- No OpenSpec spec covers the Tauri desktop shell, sidecar lifecycle, per-target data
  layers, or mobile: 67 specs exist, none matching tauri/desktop/mobile/sidecar/local-first.
  G1–G4 are entirely unspecified territory — the Spec stage must introduce new capabilities
  (e.g. desktop-shell, hybrid-data-layer) rather than extend existing ones.
- pwa-offline spec covers the service worker at requirements level but has no scenario for
  non-http(s) request schemes — the sw.js fix (supplemental change 1) should land as a
  delta extending that spec, not a new capability.
- frontend-build-tooling spec exists — the TS 7.0 migration delta belongs there.

BUILD HEALTH
- build check: PASS — cargo release build (server-full) succeeded today (13:27 & 16:31
  local); frontend `pnpm typecheck` (tsc -b) exits 0 (run during this assessment).
- known violations: settings-page.tsx (3,336 lines) et al. violate the 800-line file cap;
  frontend test coverage far below the 80% bar (29 test files / 254 source files).
- test coverage: MINIMAL (frontend) / not re-measured this session (Rust — prior phase
  certified 60% floor via cargo-llvm-cov).

CONSTRAINT CHECK
- AGENTS.md violations: NONE newly introduced; the oversized-file and coverage gaps above
  are pre-existing debt this phase explicitly targets.
- CLAUDE.md UI/UX routing constraint: BINDING for this phase — every UI change must run
  the memory-recall → UI/UX-Pro-Max → /impeccable audit+critique → frontend-design/
  ux-designer → Vercel-skills → summary pipeline before code. G5 makes this the phase's
  backbone rather than overhead.
- Strict frontend layering (components → hooks → stores → services): load-bearing for G3's
  desktop option — if pglite-oxide-in-Rust is adopted, only stores may gain invoke() paths.

GOAL PROGRESS
- G1 stable port: NOT MET — confirmed still present; ship-first candidate.
- G2 sidecar: NOT MET — embed model confirmed; one product decision (sidecar lifetime) open.
- G3 data-layer alignment: PARTIAL — web row already compliant; desktop/mobile rows missing;
  desktop engine choice is an open architecture decision.
- G4 mobile target: NOT MET — greenfield; feasibility verified via surrealkv.
- G5 impeccable program: NOT MET — baseline evidence recorded; audit not yet run.
- G6 supplemental absorption: NOT MET — 0/6; fully planned, ready to execute.
- G7 TS 7.0: NOT MET — GA verified (2026-07-08); programmatic-API caveat pushes detailed
  tooling-compat checks to Analyze; sequencing after UI waves recommended.

Sycophancy review note (S-03): the operator's framing "nothing works / UI sucks" is
supported in structure (file sizes, coverage, defect inventory) but was previously
partially refuted in function (model-field save path works; typecheck is clean; server
builds green). This phase should fix what is broken without accepting a blanket rewrite
premise — the evidence supports targeted waves, not a ground-up frontend rebuild.

ASSESSMENT COMPLETE
