## 1. Reachability investigation

- [x] 1.1 Trace `ammonia`/`crossbeam-epoch`/`rsa` reverse dependencies via
      `cargo tree -i` / `cargo tree --target all -i` to confirm exact
      provenance paths.
- [x] 1.2 Inspect `surrealdb-core`'s own `ammonia::clean` call site
      (`fnc::string::html::sanitize`) — confirm default config, no
      `math`/`annotation-xml` tags enabled (rules out `RUSTSEC-2026-0193`
      as practically exploitable here, independent of the version bump).
- [x] 1.3 Confirm UAR's own JWT usage is HMAC-only and SurrealDB auth is
      `Root`-based (not RECORD/scope JWT), ruling out `rsa`'s Marvin
      Attack via those paths.
- [x] 1.4 Trace `rsa`'s remaining reachability via `liter-llm`'s Vertex AI
      OAuth JWT-signing (`Algorithm::RS256`) — confirmed present, assessed
      against the Marvin Attack's actual threat model (network-observable
      timing oracle) and found not to fit an outbound, self-triggered
      signing flow.

## 2. Apply the fix

- [x] 2.1 `cargo update -p ammonia --precise 4.1.3` — clears
      `RUSTSEC-2026-0193`.
- [x] 2.2 `cargo update -p crossbeam-epoch` (0.9.18 → 0.9.20) — clears
      `RUSTSEC-2026-0204`.
- [x] 2.3 `rsa` 0.9.10: no version bump possible (`patched = []` in the
      advisory). Disclosed as accepted risk in `findings.md` and
      `docs/DEPENDENCY_MANAGEMENT.md`.

## 3. Verify

- [x] 3.1 `cargo check --lib` — clean (2 pre-existing unrelated warnings).
- [x] 3.2 `cargo test --lib` — 387/388 pass (1 pre-existing ignore),
      unchanged from the change-1 baseline.
- [x] 3.3 `cargo clippy --lib` — 499 warnings, same as the post-change-1
      baseline; zero new warnings (this change touches only `Cargo.lock`,
      no UAR source).
- [x] 3.4 `cargo audit` — confirmed `ammonia` and `crossbeam-epoch` no
      longer appear; `rsa`/`RUSTSEC-2023-0071` still listed as expected
      (no fix exists).

## 4. Update docs and KBD state

- [x] 4.1 Add a disposition note to `docs/DEPENDENCY_MANAGEMENT.md` for
      `rsa`'s accepted-risk status (mirrors the `kreuzberg` section added
      in change 1).
- [x] 4.2 Update
      `.kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json`
      (`change_status.surreal-memory-transitive-vulns` → DONE,
      `changes_completed` incremented).
