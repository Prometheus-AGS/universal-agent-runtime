### 2026-07-15T23:20:00Z — Analyze stage decisions (kbd-analyze)
- cand-001 pglite-oxide: ADOPT (operator-directed + skill-matrix-aligned), GATED on an
  x86_64-apple-darwin portability spike — release 0.5.1 ships no Intel-mac AOT asset and
  the primary dev machine is an Intel Mac; fallback = webview PGlite on stable origin.
  Provenance: operator (direction) + research (gate).
- cand-006 TypeScript 7.0: ADAPT (hybrid) — tsc on 7, typescript-eslint/typedoc on
  @typescript/typescript6 compat (cand-007); full migration blocked upstream
  (typescript-eslint <6.1.0 cap, typedoc rewrite pending). Sequenced after UI waves.
  Provenance: research.
- cand-005 sidecar pattern: ADOPT; sidecar-lifetime product decision escalated to operator.
- G4 mobile: all four candidates (cand-003/004/008/009) ADOPT per TJ-ARCH-MOB-001;
  child-phase split decision deferred to plan time.
- No contested stack choice (stack operator-specified); no elicitation run.

### 2026-07-15T22:35:00Z — Cross-validation against hybrid-mobile-architecture skill's reference scaffold
Reviewed the local checkout at
/Users/gqadonis/Projects/references/hybrid-mobile-architecture-skill
(now contains a verified-building apps/knowme-poc scaffold, not just docs).
Findings (full detail in analysis.md):
- CONFIRMED: liter-llm as cloud gateway (independent same-day validation from
  the reference project's own architecture revision — UAR already correctly
  positioned, no change).
- CONFIRMED + DOCUMENTED AS DELIBERATE: the reference app uses pure Tauri IPC
  (invoke()), no HTTP server anywhere. UAR's sidecar-HTTP plan (changes 1, 10)
  deliberately diverges because UAR is an existing ~100-endpoint HTTP API with
  an operator-stated requirement for OS-level port exposure to other local
  processes — a different, also-official Tauri pattern (verified via
  tauri-apps/tauri sidecar docs + community examples). Not an oversight;
  plan unchanged.
- NEW, non-actionable: config-DB pattern for provider/model settings
  (providers/model_prefs/app_settings schema) is architecturally the same
  fix class as today's Orchestrator/gpt-5.2 bug root cause. Flagged as a
  design reference for change 11's implementer; no scope change now.
- NEW, out of scope: mistral.rs (verified real GQAdonis fork) for on-device
  inference is a capability UAR's current goals don't call for. Flagged for
  the mobile child phase's own future assessment if "sovereign mobile UAR"
  becomes an explicit goal; not added speculatively (YAGNI).
- RE-CONFIRMED, NOT RE-SOLVED: pglite-oxide has no Intel-mac AOT asset;
  reference project hasn't hit or solved this either (Apple Silicon-only
  testing). change 2's spike gate stands, unchanged.
Provenance: operator-directed review + web search (tauri-apps/tauri sidecar
docs) + gh api fork verification. No elicitation needed — no contested choice.

### 2026-07-15T22:45:00Z — Desktop data layer: embedded SurrealDB is the universal baseline, pglite-oxide is an optional enhancement
Operator decision: where pglite-oxide cannot be used (Intel-mac spike FAIL,
or any future platform gap), the desktop data layer falls back to embedded
SurrealDB (surrealkv/kv-rocksdb) — the same engine UAR's server and the
mobile target (G4) already use — instead of the previously-proposed
"webview PGlite on a stable origin" fallback.

Effect: fully resolves the "second engine beside SurrealDB" concern
recorded in the original pglite-oxide adopt verdict — SurrealDB was already
a UAR dependency, so nothing new is required for change 11 to ship a
working desktop data layer regardless of the spike's outcome. Removes a
third distinct local-first engine (webview PGlite) from the architecture.
Unifies desktop's data-layer backend with mobile's. pglite-oxide, where
the spike confirms platform support, layers in as a genuine enhancement
(PG-wire compatibility, shared SQL schema with web/cloud) rather than the
change's sole viable path.

plan.md change 11 and analysis.md's pglite-oxide section updated
accordingly. Change 2 (pglite-oxide-intel-mac-spike) is no longer a hard
dependency of change 11 — it determines scope (whether pglite-oxide is
additionally wired in), not whether change 11 delivers value.
Provenance: operator.

### 2026-07-16T08:30:00Z — pglite-oxide-intel-mac-spike executed: FAIL (not a version-support gap, a real upstream bug)
Ran the spike (openspec/changes/pglite-oxide-intel-mac-spike/, archived
after implementation): pglite-oxide 0.5.1 does not compile at all on
x86_64-apple-darwin. `wasmer-wasix` 0.702.0-alpha.3 (pglite-oxide's own
transitive WASIX-runtime dependency — the only path this platform can
reach, since no `pglite-oxide-aot-x86_64-apple-darwin` crate exists) has a
non-exhaustive match against `virtual-net` 0.702.0's `NetworkError` enum
(E0004, missing `NetworkError::MessageSize` arm). This is a genuine
version-incompatibility bug between two of pglite-oxide's own pinned
dependencies, not an unverified-but-plausible platform gap — no boot or
query timings were ever obtainable because the build itself fails.

Additional non-gating context: upstream (f0rr0/pglite-oxide) has moved its
unreleased main branch to a ground-up rewrite ("Oliphaunt"), whose own
published first-release target envelope also explicitly excludes macOS
x64 — Intel Mac support isn't coming from upstream in the near term
either, independent of this specific bug.

Effect: cand-001 (pglite-oxide) in library-candidates.json downgraded from
"adopt" to "adapt" (still viable on aarch64-apple-darwin/Linux/Windows per
its AOT assets, which this spike did not test), evidence appended,
open_questions marked RESOLVED. desktop-data-layer-pglite-oxide (change
11) has no remaining reason to attempt pglite-oxide wiring on Intel Mac —
treats that platform as SurrealDB-only, full stop, no retry pending an
upstream fix. This does not change change 11's HIGH customer value or its
SurrealDB-baseline design (2026-07-15 decision above) — it only forecloses
the enhancement path on one specific platform.
Provenance: live spike execution (this change), not analysis-only.
