PLAN: uar-hybrid-app-architecture
Project: universal-agent-runtime
Date: 2026-07-15
OpenSpec available: YES
Changes to implement: 12 (+1 structural action: mobile child-phase split)

> Inputs: assessment.md, analysis.md, library-candidates.json (9 candidates:
> 8 adopt / 1 adapt), seed-analysis.md, goals.md. Ordering rationale: the P0
> data-integrity fix and the two de-risking investigations go first; UI
> remediation waves follow the audit that scopes them; the two big
> architecture changes (sidecar, desktop data layer) land once their gates
> (operator decision, Intel-mac spike) clear; TS 7.0 goes last to avoid
> toolchain churn colliding with UI waves; mobile splits into a child phase.

CHANGE LIST (ordered)

1. desktop-stable-port: replace random-port-per-launch with a stable persisted port
   - Scope: src-tauri (lib.rs) + config
   - Depends on: NONE
   - Recommended agent: Claude Code (Sonnet 5)
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: HIGH (P0 — stops silent loss of all per-origin local data on
     every desktop launch: PGlite idb://uar-threads, localStorage, SW caches)
   - Details: Default to the configured server port; on conflict, resolve once and
     persist the choice (app-config dir) so the webview origin is identical across
     launches. Remove the bind-:0 path except as a last-resort with persisted result.
     Spec: new `desktop-shell` capability, first requirement.

2. pglite-oxide-intel-mac-spike: verify portable-WASIX pglite-oxide on x86_64-apple-darwin
   - Scope: spike (standalone crate or examples/, no runtime wiring)
   - Depends on: NONE — library: cand-001
   - Recommended agent: Claude Code (Sonnet 5), run on the operator's Intel Mac
   - Est. complexity: S
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM (de-risks change 11 before any code depends on it)
   - Details: 0.5.1 ships no x86_64-apple-darwin AOT asset (analysis finding beyond
     the skill doc). Boot PgliteServer via the portable-wasix asset on this machine;
     measure cold-start and basic query latency; document PASS/FAIL + numbers in the
     change ledger. FAIL ⇒ change 11 pivots to its documented fallback.

3. admin-sw-scheme-safe-caching: scheme-guard the service worker cache writes
   - Scope: frontend (sw.js) — absorbed supplemental #1
   - Depends on: NONE
   - Recommended agent: OpenCode (Kimi K2.7 Coding)
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM
   - Details: Skip cache.put() for non-http(s) request schemes (sw.js:40-47 filter
     block; lines 64/79 are the throw sites). Lands as a delta extending the
     existing `pwa-offline` spec (non-http(s) scheme scenario).

4. admin-agent-model-warning-clarity: stop flagging "defers to system default" as broken
   - Scope: frontend (Admin Agents list/detail) — absorbed supplemental #2
   - Depends on: NONE
   - Recommended agent: Claude Code (Sonnet 5)
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: HIGH (the exact confusion behind today's bug report)
   - Details: Warning fires only when no resolution path exists (no per-agent
     override AND no registry default); otherwise neutral "Using system default".

5. impeccable-uiux-audit: run the /impeccable audit + critique program, produce the
   scored defect inventory and remediation backlog
   - Scope: frontend (read-only investigation; output is the wave-1 backlog)
   - Depends on: NONE
   - Recommended agent: Claude Code (Sonnet 5) — judgment quality here scopes all UI waves
   - Est. complexity: M
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Full CLAUDE.md UI/UX routing pipeline (memory recall → UI/UX Pro Max →
     /impeccable audit + /impeccable critique → frontend-design/ux-designer → Vercel
     skills). Known baseline: settings-page.tsx 3,336 lines (4.2x the 800 cap),
     29 test files / 254 source files, freeze report, brittleness inventory from
     assessment. Output: ranked backlog with severities feeding change 8.

6. admin-agent-provider-first-model-picker: two-step provider→model selection scoped
   to registered models
   - Scope: frontend (Edit Agent Identity tab) — absorbed supplemental #3
   - Depends on: NONE (independent of 5; already fully specified)
   - Recommended agent: Claude Code (Sonnet 5)
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (operator's primary explicitly-stated UX ask)
   - Details: Reuse the Providers page's select-provider-then-models pattern in the
     agent editor; scope the model list to GET /api/uar/providers registered models,
     closing the catalog-only-model silent-failure path.

7. governance-tool-approval-reconciliation: reconcile Governance-tab "auto" with the
   observed native__memory_* denials — investigation-first
   - Scope: api (governance engine) + frontend — absorbed supplemental #5
   - Depends on: NONE
   - Recommended agent: Claude Code (Sonnet 5)
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: MEDIUM-HIGH (trust/safety boundary; unpredictable tool denial)
   - Details: First establish which explanation holds (fail-closed default at
     policy_count=0 vs disconnected UI control); only then scope the fix. Wrong-guess
     fixes here create a worse security-UX mismatch.

8. uiux-remediation-wave-1: execute the top-severity backlog from change 5
   - Scope: frontend — absorbs supplemental #4 (edit-panel save-path verification)
   - Depends on: impeccable-uiux-audit (5); pairs with 6 landing first on the same dialog
   - Recommended agent: Claude Code (Sonnet 5)
   - Est. complexity: L
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Bounded to the wave-1 cut the audit defines (expected: settings-page
     decomposition ≤800-line files, Edit Agent save-path verification, freeze-path
     hardening, /impeccable polish + harden passes on touched surfaces). Wave 2+ only
     if the audit's severity distribution demands it — not open-ended.

9. admin-ui-freeze-diagnostics: reproduce and instrument the reported freeze —
   investigation-first
   - Scope: frontend (+ possible Worker boundary for PGlite) — absorbed supplemental #6
   - Depends on: NONE (requires operator at the keyboard for the repro sequence)
   - Recommended agent: Claude Code (Sonnet 5), live session with operator
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (a frozen admin console blocks everything else)
   - Details: Long Task PerformanceObserver + main-thread profiling during the
     operator's exact repro; confirm/refute PGlite main-thread init. No blind fixes.

10. desktop-sidecar-conversion: spawn uar-sidecar via externalBin/plugin-shell on the
    stable port
    - Scope: src-tauri + tauri.conf.json + minimal frontend boot logic — library: cand-005
    - Depends on: desktop-stable-port (1) + OPERATOR DECISION: sidecar lifetime after
      app exit (die-with-app vs persist for other OS consumers)
    - Recommended agent: Claude Code (Sonnet 5)
    - Est. complexity: M
    - Complexity score: Medium
    - Model class: medium
    - Customer value: HIGH (the operator's stated desktop model; port exposed to OS)
    - Details: Replace in-process server::start_server embed with managed uar-sidecar
      spawn; reuse the existing /readyz wait; add uar-sidecar to bundle.externalBin
      (pattern proven by the two MCP sidecars — mcp-server-filesystem and
      mcp-server-fetch, both Rust). Spec: `desktop-shell` capability.
    - Cross-validated 2026-07-15: this HTTP-sidecar approach is a deliberate,
      reasoned divergence from the hybrid-mobile-architecture skill's own
      reference app (which uses pure Tauri IPC, no HTTP server at all) — UAR's
      existing ~100-endpoint HTTP surface and the operator's OS-port-exposure
      requirement justify it; both patterns are officially Tauri-supported.
      See analysis.md's cross-validation section.

11. desktop-data-layer-pglite-oxide: move the desktop local-first data layer into Rust,
    with embedded SurrealDB as the universal baseline backend
    - Scope: src-tauri/Rust (+ stores gain invoke() paths per layering contract) —
      library: cand-001 (pglite-oxide, platform-gated enhancement) + cand-009
      (SurrealDB kv-rocksdb, baseline — already adopted for mobile)
    - Depends on: desktop-sidecar-conversion (10). pglite-oxide-intel-mac-spike (2)
      no longer hard-blocks this change (see decision below) — it now determines
      whether pglite-oxide is additionally wired in, not whether the change ships.
    - Recommended agent: Claude Code (Sonnet 5)
    - Est. complexity: L
    - Complexity score: High
    - Model class: frontier
    - Customer value: HIGH (architecture alignment; no new required engine — reuses
      UAR's existing SurrealDB dependency; unifies desktop's data layer with mobile's)
    - Details (REVISED 2026-07-15, operator decision): embedded SurrealDB (surrealkv/
      kv-rocksdb) — the same engine UAR's server and the mobile target (G4) already
      use — is the concrete backend behind the repository traits, everywhere pglite-
      oxide isn't available or hasn't been validated. This eliminates the earlier
      "second engine beside SurrealDB" concern entirely and removes the prior
      fallback ("webview PGlite on stable origin") since it introduced a *third*
      distinct local engine. pglite-oxide, where the Intel-mac spike (or any future
      platform check) confirms it works, is layered in as an optional enhancement
      (PG-wire compatibility, shared SQL schema with the web/cloud tiers) — not a
      requirement for this change to deliver a working desktop data layer. Repository
      traits mirror the web client's thread/message schema regardless of concrete
      backend; only Zustand stores may call invoke() (strict layering). Spec: new
      `hybrid-data-layer` capability, with two backend implementations behind one
      trait (surrealdb-embedded baseline, pglite-oxide enhancement).
    - Design reference (2026-07-15): the hybrid-mobile-architecture skill's
      reference app stores provider/model settings in a schema'd config DB
      (providers/model_prefs/app_settings) rather than env vars or scattered
      settings rows — the same fix class as today's Orchestrator/gpt-5.2 bug
      root cause. Worth reusing this pattern when designing this change's
      schema, though not a scope change. See analysis.md.

12. typescript-7-hybrid-migration: tsc on TS 7, API tooling on @typescript/typescript6
    - Scope: frontend toolchain — library: cand-006 (adapt) + cand-007 (adopt)
    - Depends on: uiux-remediation-wave-1 (8) — sequenced after UI churn settles
    - Recommended agent: OpenCode (Kimi K2.7 Coding), operator PR review
    - Est. complexity: M
    - Complexity score: Medium
    - Model class: medium
    - Customer value: MEDIUM (build speed ~8-12x; strict-mode hard defaults)
    - Details: typescript → 7.x for tsc -b build/typecheck; typescript-eslint and
      typedoc pinned to the @typescript/typescript6 compat surface (upstream blockers:
      typescript-eslint <6.1.0 cap, typedoc rewrite pending — trackers are the
      revisit signals). Verify vite/rolldown path (type-stripping, no TS API — low
      risk, confirm in-change). Spec: delta on `frontend-build-tooling`.

STRUCTURAL ACTION (not an OpenSpec change in this phase)
- mobile-on-device-uar child phase: G4 (Flutter + gen_ui_core FFI + SQLite/sqlite-vec +
  SurrealDB kv-rocksdb + on-device UAR via surrealkv; libraries cand-003/004/008/009)
  is greenfield and larger than this phase's remaining budget. Create it via
  /kbd-new-child once change 10 lands (the sidecar/port contract defines what mobile
  parity means). Its own assess→plan cycle runs inside the child phase.

EXECUTION ROUND ORDER
Round 1 (parallel): 1, 2, 3, 4, 5, 6  (+7 and 9 as the investigation track; 9 needs the
  operator present, schedule opportunistically)
Round 2: 8 (after 5), 10 (after 1 + operator lifetime decision)
Round 3: 11 (after 2 PASS + 10), 12 (after 8)
Then: /kbd-new-child mobile-on-device-uar (after 10)

COMMANDS TO RUN
/opsx:new desktop-stable-port
/opsx:new pglite-oxide-intel-mac-spike
/opsx:new admin-sw-scheme-safe-caching
/opsx:new admin-agent-model-warning-clarity
/opsx:new impeccable-uiux-audit
/opsx:new admin-agent-provider-first-model-picker
/opsx:new governance-tool-approval-reconciliation
/opsx:new uiux-remediation-wave-1
/opsx:new admin-ui-freeze-diagnostics
/opsx:new desktop-sidecar-conversion
/opsx:new desktop-data-layer-pglite-oxide
/opsx:new typescript-7-hybrid-migration

Sycophancy self-check
- S-02: The plan does not assume every operator claim verbatim — change 8 is bounded
  by the audit's evidence (assessment partially refuted "nothing works"), and TS 7.0
  is planned as the hybrid the research supports, not the full migration the goal
  literally requested (full migration is blocked upstream; forcing it would fail).
- S-07: Mobile (G4) is explicitly deferred to a child phase rather than inflating this
  plan; wave 2+ UI work is conditional on audit evidence, not pre-committed.
- S-03: Frictions surfaced: two changes gated on operator decisions (10: sidecar
  lifetime; 9: live repro availability); change 11 carries a real chance of shrinking
  to its fallback if the spike fails on Intel mac; TS 7.0 at 7 days post-GA is
  early-adopter risk accepted consciously and sequenced last.

PLAN COMPLETE
