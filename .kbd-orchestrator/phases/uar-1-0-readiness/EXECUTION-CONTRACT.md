# Execution contract — uar-1-0-readiness

**Read this before executing any of the six changes.** It resolves the cross-change
questions an autonomous executor cannot safely infer. The prior phase's
adversarial review returned INSUFFICIENT on six findings, and **every one was
about autonomous executability rather than correctness** — an executor that
cannot ask questions needs order, precedence, verbatim commands, and stop
conditions stated explicitly.

Authored in the Claude Code harness. Execution and reflection belong to Codex per
`.kbd-orchestrator/HARNESS-HANDOFF.md`.

## The set, and its order

Six changes in **two independent tracks**. Within a track the order is
load-bearing; the tracks share no files and may run concurrently.

### Track A — identity and tenancy (strictly serial)

0. **`fix-jwt-crypto-provider`** — **RUNS FIRST, BLOCKS EVERYTHING IN TRACK A.**
   `Cargo.toml:393` sets `jsonwebtoken = "11.0.0"` with default features, which
   enable neither `rust_crypto` nor `aws_lc_rs`. `CryptoProvider::from_crate_features()`
   therefore returns a struct whose `signer_factory`/`verifier_factory` are
   `panic!`, and nothing in `src/` calls `install_default`. Every JWT sign and
   verify panics today — `middleware.rs:48`, `api_keys.rs:265`.
   A1 task 1.2 requires the existing HS256 tests to pass unchanged; they cannot
   until this lands. **That precondition is correct and must not be amended** —
   it detected a real defect. Added 2026-08-12 after an executor correctly
   halted rather than working around it.

   **Superseding provider decision, 2026-08-13.** Standardize every UAR-owned
   `jsonwebtoken` dependency on exactly 11.0.0 with default features disabled
   and only `rust_crypto` enabled. The earlier contained spike selected AWS-LC
   using only the `server-full` dependency graph. Re-evaluation found that
   `tools/uar-jwt-proxy` already enabled RustCrypto, so a workspace build
   activated both providers and recreated the panic. RustCrypto also completed
   isolated iOS and Android builds without a native C toolchain. Because UAR is
   embeddable and downstream features remain additive, A0 additionally guards
   every UAR JWT operation with explicit, idempotent RustCrypto installation and
   acquires the process provider slot at the shared server-startup funnel.
   Final operator correction of 2026-08-14: UAR installs RustCrypto first and
   caches its own success for idempotent reuse. In `jsonwebtoken` 11.0.0,
   `CryptoProvider::install_default` delegates to `OnceLock::set`, whose error
   returns the attempted value rather than exposing the installed provider; the
   installed provider getter is crate-private. UAR therefore cannot safely
   distinguish an identical earlier installation from AWS-LC. Any process
   provider installed before UAR is a structured conflict. UAR-owned binaries
   acquire the slot at startup, and all UAR-owned dependency edges select only
   RustCrypto.

1. **`gap-02-jwks-token-verifier`** — `TokenVerifier` abstraction, JWKS lane,
   enforce `jwt_required` at the point of use, enforce `iss`/`aud`.
2. **`gap-03-a2a-tenant-partitioning`** — add the tenant claim, partition both
   A2A maps, convert the C-21 exclusion.

**Why A1 → A2 cannot be relaxed.** A2 populates `tenant_id` from a token.
Populating it from an *unverified* token is worse than having no tenant field at
all, because downstream code would then treat an attacker-controlled string as an
isolation boundary. A2 must consume a claim A1 has already authenticated.

### Track B — skills (strictly serial)

3. **`skill-builtins-on-embedded`** — register built-ins on the embedded path.
4. **`skill-scoped-governance`** — durable global/agent/conversation
   enable-disable, live effect, origin exposed.
5. **`skill-config-reconciliation`** — merge config into the database at
   startup; tombstone removed config-provisioned skills.

**Why B3 → B4 → B5 cannot be relaxed.** B4's restart tests require built-ins to
be present on the path under test, which B3 establishes. B5's restore requirement
preserves the scoped configuration B4 introduces; specifying restore before that
configuration exists leaves "preserves its prior scoped configuration"
untestable.

### Across tracks

Track A touches `src/uar/security/` and `src/uar/api/a2a/`. Track B touches
`src/uar/runtime/skills/`, `src/embedded.rs`, and `src/uar/api/skills.rs`. **No
file overlap**, so B may proceed while A is in flight. If a single executor runs
them, prefer A then B — A closes an authentication defect.

## Precedence against an existing change

`harden-jwt-defaults` (0/3 tasks, untouched since `3a54b965`, 2026-07-14)
declares the same capability, `jwt-hardening`. It is **not** part of this phase
and must not be executed here.

Both state a requirement about claim validation. They agree on behaviour, so
nothing contradicts — but two requirements over one behaviour with no ordering is
exactly what makes an executor guess:

| Concern | Governed by |
|---|---|
| JWKS lane claim validation | **`gap-02-jwks-token-verifier`** (this phase) |
| Shared-secret lane, config surface, fallback-secret refusal | `harden-jwt-defaults` (not this phase) |

`jwt-hardening` does not yet exist under `openspec/specs/`. Both changes declare
it `ADDED`; whichever archives first creates it. Archive order does not change
the outcome, because the requirement sets are otherwise disjoint.

## The pinned verification command

Quoted verbatim so no change resolves it from another change's task file.

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases -- --test-threads=1
```

Load-bearing parts, unchanged from the prior phase's baseline:

- `UAR_LIVE_INTEGRATION_BACKEND=recorded` — the in-process stub. No API keys, no
  token spend. A result taken under `live` is not comparable to the baseline.
- `--features server-full` — the certified profile. **Results transfer to no
  other profile.**
- `--test-threads=1` — every booting case is `#[serial]`.

**Baseline to beat:** `29 passed, 0 failed, 264.16s, exit 0` on `main` at
`38d41a42`, independently re-verified at phase close. A run that produces fewer
than 29 passing cases has regressed something; stop and report.

Unit tests added by these changes run under the ordinary
`cargo test --locked --no-default-features --features server-full` and do not
need the live harness.

## Tier-0 Clippy exception

Operator decision of 2026-08-13: for this phase, run:

```bash
cargo clippy --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib --no-deps
```

Success means exit code 0 and no warning introduced by the change in hand. The
existing warning baseline is acknowledged as pre-existing debt and is not part
of `uar-1-0-readiness`. Do not edit vendored dependencies or unrelated UAR
source to reduce that baseline. A warning attributable to the current change is
a failure and must be fixed within that change's permitted surface or reported
as a stop condition.

## Delivery-first verification cadence

An implementation unit is a cohesive change-level code slice, not each patch or
file write. Build the complete slice in dependency order, using formatting and
static inspection between related edits. Run Tier 0 once when that slice is
complete, then Tier 1 once the change's tests are complete. Do not repeat an
unchanged expensive command without a source change or a contract requirement.

For A1, the slice is the JWKS verifier, per-URL cache, middleware selection,
`jwt_required` enforcement, and focused tests together. Pay the test-profile
compile cost once: run the exact A0 idempotence regression first, then the A1
security group and `uar-sidecar` tests from the warmed profile. If a
test fails, narrow to that test while fixing it; do not restart the broad group
until the focused failure passes.

Negative-control restoration is exact: capture the pre-inversion source diff,
run the failing control, restore it, assert the source diff is identical, and
rerun only the affected positive assertion.

Across tracks, only implementation and isolated tests may run concurrently in
separate worktrees/build directories. Canonical KBD transitions, shared phase
artifacts, integration, and commits remain serialized.

## What counts as satisfied

A task is done when its assertion **has been observed to pass, and its negative
control has been observed to fail.** Not when the code compiles, and not when the
assertion looks correct.

Every change in this phase carries at least one fail-closed or guard requirement. A fail-closed test that has never
been seen to fail proves nothing — it is indistinguishable from a test that
always passes. **Every fail-closed assertion in this phase requires a paired
demonstration that it can fail**, with the command and its failing output
recorded.

This is not ceremony. The prior phase's most credible artifact was its L4
persistence test, and what made it credible was an env switch proving the test
could fail.

## Verification-record format

One row per requirement, in the change's own `verification.md`:

```
| requirement | assertion observed | negative control observed | command | result |
```

Rows from all six changes must be comparable, so use this format in every one.
Paste the actual command and the actual output. **Do not describe what a test "should"
produce.**

## Stop conditions

Halt and report rather than guessing. In the prior phase the executor halted for
15 hours instead of checking a box that would have misrepresented the result, and
**that halt produced a real correction to the spec.** Halting is cheaper than
guessing here too.

Each condition names a specific observable:

1. **A new crate dependency appears necessary.** The analysis concluded none is
   needed — `jsonwebtoken` 11 already provides the JWKS primitives and `reqwest`
   already has TLS. That conclusion is falsifiable; if it is wrong, say so rather
   than adding a dependency to the authentication path.
2. **A `uar-sidecar` test fails after enforcing `jwt_required`.** Expected to
   pass: the sidecar sets the flag explicitly and the defect is that the flag is
   ignored. A failure means an assumption is wrong. **Do not revert the fix and
   do not edit the sidecar** — report.
3. **A task appears to require editing `docs/SPECIFICATION.md`.** That is the
   artifact being measured against. **One sanctioned exception:**
   `skill-builtins-on-embedded` task 3.1 amends **line 445 only**. Any other
   edit to that file, in any change, is a stop.
4. **A task appears to require renaming `multi-tenant-isolation`.** Recommended
   in GAP-03's proposal, deliberately not sanctioned. Report; do not act.
5. **Tenant scoping appears to require touching runs, memory, or knowledge
   bases.** That is `fix-user-isolation-sessions-memory-kb`'s scope.
6. **The tenant claim's name conflicts with an external issuer contract.** This
   was an open question at spec time and is a real possibility.
7. **A pre-existing failure unrelated to the change in hand appears.** Report it;
   do not repair it inside these changes.

A0 adds two provider-specific conditions:

8. **A new direct crate or a package not already present in the lockfile appears
   necessary.** RustCrypto is already locked through the proxy. Stop rather than
   widening the authentication dependency surface.
9. **A UAR JWT operation cannot be routed through the provider guard within the
   A0 surface.** Stop rather than leaving a panic-capable bypass.

Track B adds four more (numbered 10–13 after the A0 additions):

10. **Any task appears to require hard-deleting a skill.** Operator decision is
   tombstone-with-restore. Reconciliation never hard-deletes.
11. **`provider_id` turns out not to distinguish config-provisioned from
   user-created skills** in some code path. The entire data-loss safety argument
   for `skill-config-reconciliation` rests on it (`fs-skills` vs `api` vs
   `builtin` vs `wasm`). If it is unreliable, stop — do not substitute a guess.
   **Operator-approved correction, 2026-08-15:** this condition fired because
   API-created skills written under the reserved `skills/dynamic/` directory
   reloaded as `fs-skills`. Before reconciliation, the filesystem loader may be
   corrected to assign `api` beneath that reserved directory and `fs-skills`
   elsewhere, and service writes to that directory may be restricted to
   `provider_id = "api"`, with cold-reload and write-side regression tests. Any
   other unreliable path remains a stop condition.
12. **Marking `fix-skills-scope-semantics` superseded appears necessary.** It is
    an operator action on another author's change.
13. **Built-in non-deletability appears to need implementing.** It already
    exists — `service.rs:390-401` rejects with `system_skill_immutable`. Consume
    it; do not rebuild it.

## Permitted surface

**Track A:**

- `Cargo.toml`, `Cargo.lock`, and `tools/uar-jwt-proxy/**` — A0 provider
  standardization, provider initialization, and focused proxy tests only.
- `src/uar/security/**` — verifier, claims, middleware.
- `src/uar/api/a2a/**` — task store, handler, grpc.
- `src/config.rs` — new optional security fields only.
- `src/server.rs` — only the `TaskStore` construction and wiring; A0 may
  additionally install RustCrypto at the start of the shared
  `start_server_with_listener` funnel. No other A0 server edit.
- **A2 compile-only expansion approved by the operator on 2026-08-14:** existing
  `UserClaims` literals in `src/server.rs`, `src/uar/admin/memory.rs`,
  `src/uar/memory/scopes.rs`, and `src/uar/api/memory_admin.rs` may add only
  `tenant_id: None`. This preserves their current behavior and does not
  authorize tenant-scoping runs, memory, or knowledge bases.

**Track B:**

- `src/uar/runtime/skills/**` — service, registry, storage providers.
- `src/uar/runtime/manager.rs` — **B4 only:** construct the skill policy universe
  from all registered skills, pass the existing conversation/session identifier
  into scoped matching, and retain the returned binding for the run lifetime. No
  other run-manager changes.
- `src/uar/domain/skills.rs` — scoped-config and tombstone shape.
- `src/uar/api/skills.rs` — expose `origin`.
- `src/embedded.rs` — built-in registration on the embedded path.
- `src/server.rs` — startup reconciliation wiring only.
- `docs/SPECIFICATION.md` — **line 445 only**, in `skill-builtins-on-embedded`
  task 3.1. Nowhere else in the phase.

**Both:** `tests/**` — new tests, plus converting the C-21 exclusion.

Anything outside this is a stop condition, including tempting adjacent cleanups.
Per repo rules: minimum change that solves the problem; do not refactor adjacent
working code.

## Reporting constraints

Carried forward from the prior phase and still binding:

- **No aggregate percentage.** No runtime-level verdict.
- Per-requirement results only, each with its stated limit.
- Runtime results are scoped to `server-full`. A0 may additionally report its
  two explicit `embedded-mobile` target checks separately; neither target result
  transfers to another profile or target.
- Commit per change. **Do not push. Do not open a PR.** The authoring harness
  reconciles and verifies independently.

## GAP-05: what the spec says is wrong, and this contract said so too

**Retraction.** An earlier revision of this contract stated that built-in skills
are registered *in-memory only*, and that this is why the embedded path has no
built-ins. **That was true when `add-skill-system-submodule` was authored and is
false on current `main`.** Commit `fdd69a2f` (*"persist builtin skills when no
embedder is configured"*) changed it. Verified in code rather than from doc
comments: `registry.rs:69-99` — `register` writes through `db.save_skill`.

`docs/SPECIFICATION.md:445` is wrong in three ways, and an executor planning from
it would build the wrong thing:

| Spec claim | Verified on `main` |
|---|---|
| `register_builtins` called only from `server.rs:436` | **Two** sites, `server.rs:454` and `:517`; neither line matches |
| Built-ins are not persisted | **They are** — `registry.rs:69-99` |
| Embedded boots with an empty registry, "capability at 0%" | **Only on a fresh database.** `embedded.rs:365-371` registers a `DatabaseStorageProvider` and calls `initialize()` |

The real defect is narrow: `embedded.rs` never calls `discover_builtin_skills()`
/ `register_builtins`, so on a *fresh* embedded database no built-in ever enters.
That is what `skill-builtins-on-embedded` fixes, and it is the only change in
this phase permitted to edit `SPECIFICATION.md` — one line, task 3.1.

**Operator decisions of 2026-08-12** shaped Track B and are recorded here so the
executor does not re-open them:

1. Built-ins live in permanent storage and are never deletable.
2. All skills are configurable at global, agent and conversation scope, durably
   and with live effect.
3. Configuration files reconcile into the database at startup.
4. **Removal by reconciliation is a tombstone with restore, never a hard
   delete.** This is the only irreversible operation in the subsystem and file
   absence has innocent causes — a mis-mounted volume, a partial checkout.

`skill-scoped-governance` **supersedes** `fix-skills-scope-semantics` (0/5) and
adopts its `skill-governance` capability name. Do not execute both. Marking that
change superseded is an operator action on another author's work — a stop
condition, not a task.
