# Plan — uar-1-0-readiness

Authored 2026-08-12 in the Claude Code harness. Execution and reflection belong
to Codex per `.kbd-orchestrator/HARNESS-HANDOFF.md`.

**Baseline:** `origin/main` at `88c38015`.
**Contract:** `EXECUTION-CONTRACT.md` in this directory, symlinked into all six
changes. It is not optional reading.

## Scope, as delivered by the spec stage

| # | Change | Capability | Validates |
|---|---|---|---|
| A0 | `fix-jwt-crypto-provider` | `jwt-hardening` (ADDED) | strict pending re-validation after RustCrypto supersession |
| A1 | `gap-02-jwks-token-verifier` | `jwt-hardening` (ADDED) | strict ✅ |
| A2 | `gap-03-a2a-tenant-partitioning` | `tenant-isolation` (new) | strict ✅ |
| B3 | `skill-builtins-on-embedded` | `skill-builtin-availability` (new) | strict ✅ |
| B4 | `skill-scoped-governance` | `skill-governance` (new) | strict ✅ |
| B5 | `skill-config-reconciliation` | `skill-config-reconciliation` (new) | strict ✅ |

**Widened 2026-08-12 on operator direction** — the phase now aims at a fully
functional skill subsystem, not just the three adoption-blocking gaps. GAP-05
became Track B once grounding showed the spec's description of it was wrong in
three ways (see the contract's retraction section).

## Sequence

```
Track A   fix-jwt-crypto ─> gap-02 ─────────> gap-03
          (verifier, JWKS)   (tenant claim, partition, convert C-21)

Track B   skill-builtins ──> skill-scoped ──> skill-config
          (embedded path)    (governance)     (reconciliation)
```

Serial within each track; the tracks share no files and may run concurrently.

**A0 decision revision, 2026-08-13.** The workspace standard is
`jsonwebtoken` 11.0.0 with RustCrypto, not AWS-LC. The selection is centralized
in `[workspace.dependencies]`, and runtime JWT operations explicitly install
the chosen provider so a downstream additive feature cannot turn provider
selection back into an implicit panic.

**A0 ownership clarification, 2026-08-14.** UAR acquires the process provider
slot at the shared server-startup funnel and caches that successful RustCrypto
installation. Any provider initialized before UAR fails closed, including an
indistinguishable RustCrypto installation; exact-v11 exposes no public accessor
that could prove the prior provider's identity.

- **A1 → A2**: A2 consumes a type A1 introduces, and populating a tenant from an
  unverified token is worse than having no tenant field.
- **B3 → B4 → B5**: B4's restart tests need built-ins present on the path under
  test; B5's restore requirement preserves the scoped configuration B4 adds.
  Specifying restore first leaves "preserves prior scoped configuration"
  untestable.

## Exit criteria

The phase is done when **all** hold:

1. All six changes' tasks complete, each assertion **observed** to pass.
2. Every fail-closed assertion has a **negative control observed to fail**, with
   command and output recorded.
3. The pinned command produces **≥ 29 passing cases, 0 failed** — no regression
   against the `38d41a42` baseline.
4. `excluded_c21_tenant_isolation_no_cross_read_surface` no longer exists as an
   exclusion; a real two-tenant denial test stands in its place.
5. `openspec validate <change> --strict` passes for all six after execution.
6. A `verification.md` per change in the contract's row format.
7. A fresh embedded database yields built-in skills; a scoped disable survives
   restart and takes effect live; a config-removed skill is tombstoned and
   restorable, and no API-created or built-in skill is ever tombstoned.

Not exit criteria: "code compiles", "review looks right", any aggregate score.

## Verification tiers

Per `CLAUDE.md`, and not earlier than their point:

- **Tier 0**, every implementation unit — `cargo check --locked --no-default-features --features server-full`
  plus `cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps`. (Scope clippy to the package;
  `--all-targets` is blocked by ~140 pedantic errors in a vendored submodule.)
- **Tier 1**, unit complete — the change's own unit tests.
- **Tier 2**, phase completion — the pinned live command, verbatim.
- **Tier 3** — not reached in this phase; no milestone or release here.

## Handoff to Codex

Per `HARNESS-HANDOFF.md`, everything Codex needs must be on disk and in git
before it starts, because it does not share this conversation.

**Prompt shape** — point at the contract, do not restate it:

- Work in a worktree off current `main` (`scripts/worktree-new.sh`, which places
  it under `~/.claude/worktrees/`, never inside the repo tree).
- **Read `EXECUTION-CONTRACT.md` first** — named as non-optional.
- Changes in track order: Track A (`gap-02-jwks-token-verifier` →
  `gap-03-a2a-tenant-partitioning`), Track B (`skill-builtins-on-embedded` →
  `skill-scoped-governance` → `skill-config-reconciliation`). Serial within a
  track; the tracks are file-disjoint.
- Goal condition: the seven exit criteria above.
- Verification command verbatim from the contract, with every load-bearing flag
  explained there.
- Tier discipline as above; the tier-guard hook will block a Tier 2 run before
  its point.
- Permitted surface (per track) and the eleven stop conditions, both in the contract.
- Reporting constraints: no aggregate percentage, no runtime verdict, results
  scoped to `server-full`.
- **Commit per change. Do not push. Do not open a PR.**

**Worktree note:** fresh worktrees need
`git submodule update --init --recursive` before `cargo` will resolve, or the
failure presents as a confusing manifest error.

## What the authoring harness owes after Codex reports done

From the reconciliation checklist, all of it, not the convenient parts:

- [ ] Fetch; compare Codex's branch against **`origin/main`**, not the local ref.
      A stale local `main` produced a wrong "nothing merged" call last phase.
- [ ] **Re-run the pinned command independently** on a fresh checkout. Reading
      the executor's committed artifacts is not verification.
- [ ] **Diff the merged spec against the reviewed spec; surface every delta.**
      The executor is not obliged to flag its own scope changes, so this belongs
      on the authoring side. Last phase it added five exclusions and a repo-wide
      CI prohibition, both defensible, neither reviewed.
- [ ] Update `progress.json` from real state, not from this plan.
- [ ] Write `reflection.md` leading with the delta.
- [ ] Gate any worktree removal on unique `.prometheus` content, **not**
      `git status` — a clean status proves nothing about ignored files.

## Risks

| # | Risk | Handling |
|---|---|---|
| R-1 | Tenant claim name conflicts with an external issuer contract | Stop condition 6. Unresolved at spec time; stated rather than assumed away |
| R-2 | Enforcing `jwt_required` breaks a sidecar test | Stop condition 2. Expected to pass; a failure means an assumption is wrong, not that the fix is |
| R-3 | `harden-jwt-defaults` is worked concurrently by someone else | Precedence rule in the contract. Both edit `middleware.rs` |
| R-4 | The overlap scan missed a change | **Real and unmitigated.** ~190 active changes; I scanned only those matching security, a2a, and skill-registration terms. A miss shows up as a merge conflict, not a silent wrong result |
| R-5 | ~~GAP-05's hold leaves no parallel work~~ | **Resolved by the widening.** Track B is file-disjoint from Track A |
| R-6 | B5 tombstones a user-created skill through a `provider_id` path I did not check | Stop condition 9 plus a dedicated negative control (B5 task 4.8). **This is the phase's only data-loss risk** |
| R-7 | `skill-scoped-governance` supersedes an unstarted change someone else authored | Stated in its proposal; marking it superseded is a stop condition, not a task |
| R-8 | A downstream crate enables another `jsonwebtoken` provider additively | A0 acquires RustCrypto at the shared server-startup funnel and fails closed if any provider already owns the process slot |
| R-9 | Native crypto toolchain stalls are mistaken for backend correctness evidence | RustCrypto target probes and full profile checks run with the compiler cache disabled when the observed stall recurs |

## Open items carried out of this phase

1. **Mark `fix-skills-scope-semantics` superseded.** Operator action;
   `skill-scoped-governance` absorbs its scope and capability name.
2. **`add-skill-kind-and-origin` has 3 tasks left** (8/11). It defines
   `SkillOrigin`, which Track B consumes. Not a blocker — the enum and the
   delete guard already exist — but it should be finished or closed.
3. **Rename `multi-tenant-isolation` → `user-data-isolation`** — recommended,
   deliberately unsanctioned, stop condition 4.
4. **`SPECIFICATION.md` citation drift beyond line 445** — GAP-03 cites
   `task_store.rs:16` (actual `:17-21`). Only line 445 is sanctioned for edit in
   this phase; correct the rest when the file is next touched deliberately.
