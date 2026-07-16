# Goals

- Fix the P0 desktop data-loss bug: replace random-port-per-launch in src-tauri/src/lib.rs with a stable, persisted localhost port so IndexedDB/localStorage/SW origins survive restarts
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing uar-sidecar binary via externalBin/plugin-shell, exposing its fixed port to the operating system
- Align the local-first data layer to the hybrid-mobile-architecture skill's per-target matrix: web keeps PGlite idb://, desktop moves the data layer into Rust via pglite-oxide, mobile uses SQLite+sqlite-vec via gen_ui_core FFI (pglite-oxide has no iOS/Android support per the skill's corrected doc)
- Establish the mobile (Flutter) target where UAR runs entirely on-device using its embedded SurrealDB (surrealkv) persistence, per TJ-ARCH-MOB-001
- Run /impeccable audit and /impeccable critique across the React frontend to produce a scored UI/UX defect inventory (brittleness, stalls, freeze paths, admin console UX), then execute the prioritized fixes with /impeccable polish/harden
- Absorb the 6 supplemental Admin/Agents UI changes from uar-grade-a-upgrade-2026-07's supplemental plan (sw-scheme-safe-caching, model-warning-clarity, provider-first-model-picker, edit-panel-verification, governance-reconciliation, freeze-diagnostics) as seed work items for this phase
- Migrate the frontend toolchain to TypeScript 7.0 (native compiler), verifying current release status, ecosystem compatibility (Vite/rolldown, ESLint, vue-tsc-equivalents), and migration path from 5.9.3 before committing per dependency-verification rules 22/23
