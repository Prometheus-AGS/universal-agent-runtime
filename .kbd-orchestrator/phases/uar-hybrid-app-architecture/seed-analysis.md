# Seed Analysis — uar-hybrid-app-architecture

> Written 2026-07-15 at phase creation, from a live operator-directed
> investigation. This is assess/analyze input — findings here were verified
> against the running system and the codebase, not assumed. `/kbd-assess`
> should treat this as prior context and re-verify only what it needs.

## Operator intent (verbatim goals distilled)

The web application must work (1) served as web from the UAR itself, and
(2) inside a Tauri desktop app that starts the UAR as a **sidecar** and
exposes its port to the operating system, and (3) eventually on mobile
where instances of the UAR run entirely on-device. Additionally: the UI/UX
is "brittle, stalls, has architectural problems" — run the `/impeccable`
skill family (audit, critique, then polish/harden) over the React frontend;
address local-first data management and multi-device/environment support;
and migrate to TypeScript 7.0.

## Governing references

- **hybrid-mobile-architecture skill** (now public):
  `github.com/Know-Me-Tools/hybrid-mobile-architecture-skill` — the
  decision authority is its `references/arch-standard.md` (TJ-ARCH-MOB-001).
  Its `docs/pglite-oxide-tauri-hybrid.md` (corrected revision, 2026-07-15)
  carries the authoritative per-target data matrix used below.
- **pglite-oxide**: `github.com/f0rr0/pglite-oxide`, crate 0.5.1 —
  PGlite (PostgreSQL 17.5 WASI guest) hosted inside a Rust process,
  exposing real PG wire protocol; SQLx/tokio-postgres connect unchanged.
  **Verified platform support: Linux x64/arm64, macOS arm64, Windows x64
  only. No iOS/Android.** ("Oliphaunt" successor is pre-release; do not
  architect against it.)

## Verified findings (2026-07-15, this codebase)

### F1 — P0 data-loss bug in the current Tauri shell
`src-tauri/src/lib.rs::resolve_localhost_port` binds `127.0.0.1:0` (random
free port) on every launch unless `TAURI_LOCALHOST_PORT` is set, then
navigates the webview to `http://127.0.0.1:<port>`. IndexedDB, localStorage,
and service-worker caches are per-origin, and origin includes the port —
so every desktop launch orphans the PGlite thread DB (`idb://uar-threads`),
wipes localStorage (theme, onboarding), and strands SW caches per dead
port. Server-side SurrealDB data survives, which masks the bug.
**Fix direction:** stable, persisted port (default from config, persist the
resolved choice; fall back deterministically).

### F2 — Embedded server vs. the operator's sidecar requirement
The current shell **embeds** the UAR (`server::start_server` spawned on
Tauri's async runtime). The operator wants the existing `uar-sidecar`
binary spawned as a Tauri sidecar (`bundle.externalBin` + plugin-shell,
already a frontend dependency), with the port exposed to the OS so other
local apps can use the runtime. The `/readyz` wait loop in lib.rs already
exists and transfers directly. Current `externalBin` only ships two MCP
servers; `tauri.conf.json` has `frontendDist: ./static`, no CSP set.

### F3 — Per-target data-layer matrix (from the skill's corrected doc)
| Target | Relational/vector | Graph | Notes |
|---|---|---|---|
| Web | PGlite 0.5.4 `idb://` + pgvector | SurrealDB `kv-indxdb` | **Frontend already complies** (`frontend/src/lib/db.ts`, `idb://uar-threads`) |
| Desktop (Tauri) | pglite-oxide 0.5.1 in the Rust layer; WebView never touches the DB; stores call `invoke()` | SurrealDB `kv-rocksdb` | Adds a second engine beside UAR's SurrealDB — needs an explicit decision (justified by shared PG schema with web + Electric-sync future) |
| Mobile (Flutter) | SQLite + sqlite-vec via `gen_ui_core` FFI — **not pglite-oxide** | SurrealDB `kv-rocksdb` | UAR-on-device is viable because UAR's persistence is embedded SurrealDB (surrealkv, pure Rust) |
| Cloud | Postgres/Supabase + pgvector | SurrealDB server | |
Embedding dims standardized at 384 (or truncated-768) so vectors replicate
across engines.

### F4 — Frontend fundamentals that already work in our favor
- All service calls use **relative URLs** (`fetch("/api/...")`) — no
  hardcoded hosts anywhere in `frontend/src`. Works for web and for the
  navigate-to-localhost desktop model.
- `@tauri-apps/api` + `@tauri-apps/plugin-shell` already in package.json.
- Known frontend defects feeding the /impeccable work: service worker
  lacks a URL-scheme guard (`sw.js:64,79` throw on `chrome-extension://`);
  PGlite (multi-MB WASM) initializes app-wide via DbProvider including on
  admin routes; an unexplained UI freeze report (not reproduced; PGLite
  main-thread init is a suspect, unconfirmed); Groq/llama tool-call
  flakiness surfaces as error cards (upstream, now degrades gracefully).

### F5 — Absorbed supplemental changes (from uar-grade-a-upgrade-2026-07)
Six planned-but-unstarted changes transfer into this phase as seed work
items (full details in that phase's plan.md "Supplemental Plan" section
and assessment.md):
1. `admin-sw-scheme-safe-caching` (S, confirmed root cause)
2. `admin-agent-model-warning-clarity` (S)
3. `admin-agent-provider-first-model-picker` (M — operator's primary UX ask)
4. `admin-agent-edit-panel-verification` (M, depends on 3)
5. `governance-tool-approval-reconciliation` (L, investigation-first)
6. `admin-ui-freeze-diagnostics` (M, investigation-first; overlaps F4)

### F6 — TypeScript 7.0 migration (verify before committing)
Repo is on TS 5.9.3 (per CLAUDE.md). TS 7.0 is the native (Go) compiler
line. Per dependency-verification rules 22/23, the assess/analyze stages
MUST verify against current official sources: release status as of now,
compatibility with this repo's toolchain (Vite/rolldown bundler — note
`rolldown-runtime` chunks in the built assets — ESLint flat config,
type-aware lint rules, any API-extractor/typedoc usage in the docs
pipeline), and the 5.9 → 7 migration path. Do not assume training-era
knowledge is current; this is a goal with a verification gate, not a
foregone conclusion.

## Suggested change shape (input to /kbd-plan, not binding)

1. Stable-port fix (S, P0 — shippable alone, immediately stops data loss)
2. Sidecar conversion (M — externalBin + plugin-shell + readyz + fixed port)
3. /impeccable audit + critique sweep → scored defect inventory (M,
   investigation output feeds 4)
4. UI/UX remediation waves from the inventory, absorbing supplemental
   changes 1–4 (sizes per inventory)
5. Desktop data-layer decision + implementation (pglite-oxide in Rust vs.
   keep webview PGlite on stable origin) — spec-first (L)
6. Mobile target bootstrap per TJ-ARCH-MOB-001 (gen_ui_core seam, Flutter
   surface, on-device UAR via surrealkv) — likely its own sub-phase (XL)
7. TS 7.0 migration behind the verification gate (M, after 3–4 so lint/
   typecheck churn doesn't collide with the UI waves)
8. Supplemental 5–6 (governance reconciliation, freeze diagnostics) run as
   the investigation track alongside.

## Constraints carried forward

- UI/UX work routing in CLAUDE.md is mandatory for every UI change in this
  phase: memory recall → UI/UX Pro Max → `/impeccable audit` +
  `/impeccable critique` (plus work-specific commands) → Anthropic
  frontend-design/ux-designer → Vercel skills → summary → then code.
- Strict frontend layering (components → hooks → stores → services) is
  load-bearing for the desktop model: only stores may gain `invoke()`
  call paths if the pglite-oxide-in-Rust option is chosen.
- Paused phase `perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion`
  still awaits an operator resume decision; its certification changes must
  rerun after this phase lands source changes.
