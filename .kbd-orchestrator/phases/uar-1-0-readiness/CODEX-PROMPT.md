# Codex prompt — uar-1-0-readiness

Paste the block below into the Codex desktop harness. It points at the contract
rather than restating it, per `.kbd-orchestrator/HARNESS-HANDOFF.md`.

---

Work in the `universal-agent-runtime` repo. Create a worktree off current `main`:

```bash
scripts/worktree-new.sh uar-1-0-readiness
git submodule update --init --recursive
```

The worktree lands under `~/.claude/worktrees/`, never inside the repo tree. The
submodule init is required — without it `cargo` fails with a confusing manifest
error.

**Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md` first.
This is not optional.** It carries the execution order, the precedence rule
against an existing change, the verbatim verification command, what counts as
satisfied, the permitted surface (per track), and eleven stop conditions. It is
symlinked into all five change directories.

Execute these five changes, committing after each. They form **two tracks**.
Order within a track is load-bearing; the tracks share no files.

**Track A — identity and tenancy (serial):**

1. `openspec/changes/gap-02-jwks-token-verifier`
2. `openspec/changes/gap-03-a2a-tenant-partitioning`

**Track B — skills (serial):**

3. `openspec/changes/skill-builtins-on-embedded`
4. `openspec/changes/skill-scoped-governance`
5. `openspec/changes/skill-config-reconciliation`

A2 consumes a type A1 introduces, and populating a tenant identity from an
unverified token would make an attacker-controlled string an isolation boundary.
B4's restart tests need the built-ins B3 establishes; B5's restore requirement
preserves the scoped configuration B4 adds. **Do not reorder within a track.**
If running everything sequentially, do Track A first — it closes an
authentication defect.

**Done means all seven exit criteria in
`.kbd-orchestrator/phases/uar-1-0-readiness/plan.md` hold**, notably:

- every assertion **observed** to pass, not merely written;
- every fail-closed assertion paired with a **negative control observed to
  fail**, with command and output recorded — an untested fail-closed assertion is
  indistinguishable from one that always passes;
- the pinned command yields **≥ 29 passing, 0 failed** (no regression against the
  `38d41a42` baseline);
- the C-21 exclusion is replaced by a real two-tenant denial test;
- `openspec validate <change> --strict` passes for all five;
- a `verification.md` per change in the contract's row format;
- a fresh embedded database yields built-in skills; a scoped disable survives
  restart and takes effect live; a config-removed skill is tombstoned and
  restorable, and **no API-created or built-in skill is ever tombstoned**.

Verification command, verbatim from the contract:

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases -- --test-threads=1
```

Tier discipline per `CLAUDE.md`: Tier 0 (`cargo check` + `cargo clippy -p
universal-agent-runtime`) on every edit; Tier 1 unit tests when a unit is
complete; Tier 2 (the command above) at phase completion only. The tier-guard
hook will block a Tier 2 run before its point. Scope clippy to the package —
`--all-targets` is blocked by pedantic errors in a vendored submodule.

Permitted surface is listed in the contract. Anything outside it is a stop
condition, including adjacent cleanups that look worth doing.

**Stop and report rather than guessing** when any of the contract's eleven stop
conditions fires. Halting is the correct behaviour: in the prior phase an
executor halted for 15 hours instead of checking a box that would have
misrepresented the result, and that halt produced a real correction to the spec.
Three conditions are especially likely here — a `uar-sidecar` test failing after
`jwt_required` is enforced, a new crate dependency appearing necessary, and
`provider_id` turning out not to distinguish config-provisioned from
user-created skills. That last one is the phase's only data-loss risk: it is the
entire safety argument for reconciliation, so if it is unreliable, **stop rather
than substituting a guess**.

Reconciliation never hard-deletes. Removal is a tombstone with restore, by
operator decision.

Reporting constraints: **no aggregate percentage, no runtime-level verdict.**
Per-requirement results only, each with its stated limit. All results are scoped
to the `server-full` profile and transfer to no other.

**Commit per change. Do not push. Do not open a PR.** The authoring harness
reconciles and verifies independently on a fresh checkout.
