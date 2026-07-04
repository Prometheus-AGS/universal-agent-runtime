PLAN: uar-security-deps-and-hygiene
Project: universal-agent-runtime
Date: 2026-07-04
OpenSpec available: YES
Changes to implement: 10

## Framing

All 10 candidate changes are independent of each other — unlike
`uar-spec-v2-and-polish`'s CH-12→13→{14,15} chain, there is no
technical dependency graph here. Ordering below is by **risk**, not
dependency: small, low-blast-radius fixes first to bank quick wins and
reduce total open surface, then the two dependency upgrades — each
sequenced on its own, not run in parallel with the other, purely to
avoid confounding a regression from one with the other (both touch
broad build/test surface: `rmcp` touches MCP client/server code paths,
`surrealdb` touches persistence + all 12 migrations).

## CHANGE LIST (ordered)

### Round 1 — quick, independent, low risk

1. **dependabot-yml**: add `.github/dependabot.yml` for version-update
   automation
   - Scope: ci
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (prevents this 96-alert backlog silently
     reaccumulating; not user-facing but ops-critical)
   - Details: Config-only change (npm + cargo ecosystems, weekly
     schedule, grouped minor/patch updates). No code touched.

2. **fix-uar-integration-test**: add the 8 missing `Skill` struct
   fields in `tests/uar_integration.rs:430`
   - Scope: tests
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (unblocks `cargo check --tests` running
     clean end-to-end for the first time in this project's tracked
     history)
   - Details: Mechanical — match the `Skill` struct's current field
     list (`authors`, `compatibility`, `language`, + 5 more) and fill
     with sensible test values, following the pattern of neighboring
     `Skill` literals already in that file if any exist.

3. **fix-bdd-test-path**: fix `tests/bdd.rs`'s nested `#[path]`
   resolution
   - Scope: tests
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (same as above — unblocks a second target)
   - Details: The nested `mod live { #[path = "integration/live/harness.rs"] ... }`
     resolves relative to `tests/live/` (since the outer `mod live` has
     no `#[path]` of its own), not `tests/`. Either give `mod live` an
     explicit path, or flatten to direct `#[path = "tests/integration/live/harness.rs"]`-
     style paths on each inner item.

4. **fix-waypoint-stage-schema**: fix `write-position-reminder.sh`'s
   `.stage`/`.status` mismatch at the source
   - Scope: tooling (`shared/scripts/write-position-reminder.sh` or the
     waypoint schema/writers)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW (internal KBD tooling only, but prevents a
     silent `Stage: unknown` regression for every future phase)
   - Details: Either make every waypoint-writing skill/script populate
     `.stage` consistently (this project's actual convention, already
     hand-patched twice this phase), or change the reader to prefer
     `.status` when `.stage` is absent — pick one, don't leave both
     conventions live.

5. **artifact-refiner-gate-decision**: resolve the 4th+ phase of
   carried QA-gate debt with an explicit decision
   - Scope: process/docs (likely `.kbd-orchestrator/` convention docs,
     no application code)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (stops an indefinitely-recarried debt item
     from silently repeating every phase)
   - Details: The tool is confirmed unavailable in this environment
     (`ToolSearch` returned no matches). This change is a **decision**,
     not a build: either document that this project's KBD flow
     verifies changes via `cargo check`/`test`/`clippy` directly and
     formally retires the artifact-refiner gate requirement from this
     project's contract, or file a concrete, actionable follow-up (e.g.
     "requires provisioning X tool/service") instead of an open-ended
     "automate this" that has now failed to happen 4 times.

6. **npm-deps-triage**: trace `dompurify`/`jsonwebtoken` reachability
   and patch what's realistic
   - Scope: frontend/deps
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW (security-hygiene, not user-facing)
   - Details: `dompurify@3.4.7` is confirmed present transitively;
     trace its actual importer (`pnpm why dompurify` once
     `node_modules` is installed, or manual `pnpm-lock.yaml` graph
     walk) and bump the importer if a patched dompurify version is
     reachable. `jsonwebtoken` wasn't found in either lockfile checked
     during assessment — check git submodules
     (`prometheus-entity-management`, skill-pack) before concluding
     it's a stale/removed alert. **Disclosed scope cut**: this change
     may legitimately end in "traced, no action possible without a
     submodule-side fix" rather than a clean patch — that's an
     acceptable outcome, not a failure, provided it's disclosed rather
     than silently dropped.

### Round 2 — independent, moderate effort

7. **wasmtime-disposition**: bump or explicitly document residual risk
   - Scope: deps
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S-M · Complexity score: Medium · Model class: medium
   - Customer value: LOW (opt-in feature, `wasm-runtime` not in
     `default`, so exposure is limited to deployments that explicitly
     enable it)
   - Details: Check whether a `wasmtime`/`wasmtime-wasi` version exists
     upstream that fixes the 2 critical + 1 high alerts without a
     breaking API change to this repo's (currently minimal) WASM
     integration. If a clean bump exists, take it (with the
     `wasm-runtime` feature enabled for the verification build only).
     If not, write the residual-risk disposition explicitly (where,
     matching this phase's other decision docs) rather than silently
     carrying it forward again.

8. **run-hot-path-bench**: actually execute `benches/hot_path.rs`
   - Scope: benches
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW (internal perf validation, not user-facing)
   - Details: `cargo check --benches` first (confirm it still compiles
     — nobody has checked since it was written), then `cargo bench
     --bench hot_path` for a real baseline. Record the numbers
     somewhere durable (a comment in the bench file, or a short note in
     this change's proposal.md) so "run it" doesn't silently regress to
     "never run" again next time someone touches the hot path.

### Round 3 — dependency upgrade, own checkpoint (sequenced, not parallel with Round 4)

9. **rmcp-pin-bump**: bump the `rmcp` git-rev pin past the DNS
   rebinding fix
   - Scope: deps | mcp | api
   - Depends on: NONE (sequenced here deliberately — see Framing above)
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: medium
   - Customer value: HIGH (fixes a high-severity vulnerability in the
     core, non-optional MCP SDK)
   - Details: Identify the specific upstream commit that fixes the
     Streamable HTTP DNS-rebinding issue (bisect between the current
     pin `085470025f6...` and `HEAD` `bdf0c32e8c1...` if the fix commit
     isn't obvious from the advisory), bump the `rev` in `Cargo.toml`,
     `cargo update -p rmcp`, then run the full test suite plus any
     MCP-specific integration tests (`tests/integration/live/` MCP
     paths, `mcp.json`-driven tool discovery) as this change's own
     checkpoint before moving on.

### Round 4 — dependency upgrade, own checkpoint (highest blast radius, sequenced last)

10. **surrealdb-upgrade**: bump `surrealdb` from pinned `=3.0.5` to
    latest compatible 3.x
    - Scope: deps | persistence | migrations | all
    - Depends on: NONE (sequenced last deliberately — see Framing above)
    - Recommended agent: Claude Code
    - Est. complexity: M-L · Complexity score: High · Model class: frontier
    - Customer value: HIGH (fixes high-severity session-hijack/
      privilege-escalation CVEs in the default persistence backend)
    - Details: Follow `docs/DEPENDENCY_MANAGEMENT.md`'s existing
      upgrade SOP. Check the 12 SurrealDB migrations for any breaking
      schema/query syntax changes between 3.0.5 and the target version
      before bumping `Cargo.toml`. This is the highest-blast-radius
      change in this phase — full `cargo test --lib` +
      `cargo test --test integration` + a live-server smoke check
      (boot with the embedded SurrealKV backend, confirm basic CRUD)
      as this change's own dedicated checkpoint. **Disclosed risk**: if
      3.1/3.2 introduce a breaking change this repo's migrations can't
      absorb cleanly in one pass, the responsible outcome is to stop,
      document the blocker, and re-carry this as debt rather than force
      a partial/unverified upgrade — do not compromise on the
      checkpoint to hit a "finish everything" target.

## EXECUTION ROUND ORDER

- **Round 1 (parallel)**: dependabot-yml, fix-uar-integration-test,
  fix-bdd-test-path, fix-waypoint-stage-schema,
  artifact-refiner-gate-decision, npm-deps-triage
- **Round 2 (parallel)**: wasmtime-disposition, run-hot-path-bench
- **Round 3 (sequenced, own checkpoint)**: rmcp-pin-bump
- **Round 4 (sequenced, own checkpoint, last)**: surrealdb-upgrade

Implementation-first within Round 1 and Round 2 (batch the 6 + 2 small
changes, one verification checkpoint per round), then a dedicated
checkpoint for each of Round 3 and Round 4 individually given their
real regression risk — consistent with this project's standing
implementation-first/test-at-checkpoints preference, scaled to each
change's actual risk rather than applied uniformly.

## COMMANDS TO RUN

```
/opsx:new dependabot-yml
/opsx:new fix-uar-integration-test
/opsx:new fix-bdd-test-path
/opsx:new fix-waypoint-stage-schema
/opsx:new artifact-refiner-gate-decision
/opsx:new npm-deps-triage
/opsx:new wasmtime-disposition
/opsx:new run-hot-path-bench
/opsx:new rmcp-pin-bump
/opsx:new surrealdb-upgrade
```

Consistent with this project's actual established practice (confirmed
via `git log` on `uar-spec-v2-and-polish`'s plan commit `16c1aa3`,
which did not pre-create any `openspec/changes/` directories), these
`openspec/changes/<id>/proposal.md` + `tasks.md` pairs are written
**per-change at execute time**, not pre-scaffolded during planning.

## Sycophancy self-check

- S-02: `npm-deps-triage`'s scope cut (may end in "traced, no action
  possible") and `surrealdb-upgrade`'s disclosed risk (may need to stop
  and re-carry if migrations don't absorb cleanly) are both stated up
  front, not discovered later and smoothed over.
- S-07: no scope creep beyond the 10 changes `assessment.md` already
  identified — this plan does not add new work items.
- S-03: at least 3 explicit trade-offs/deferrals stated above
  (`npm-deps-triage`'s possible dead end, `wasmtime-disposition`'s
  residual-risk fallback, `surrealdb-upgrade`'s stop-and-re-carry
  option).

PLAN COMPLETE
