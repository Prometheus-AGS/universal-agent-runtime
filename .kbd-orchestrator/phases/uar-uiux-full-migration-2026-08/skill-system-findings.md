# Prometheus Skill System — Findings from the UAR Proving Run

**Purpose.** UAR is the test fixture; the Prometheus skill system and KBD are what is
under test. This file records defects found in the *tooling*, the fix applied, and the
evidence that the fix worked. Findings about UAR's frontend belong in `assessment.md`.

**Method.** Find defect → fix in `prometheus-skill-pack` → rebuild → reinstall/reload →
re-run the failing case → prove → continue.

---

## F-001 — `prometheus kbd` writes are impossible under the managed configuration

**Severity:** CRITICAL — silently destroys recorded work.
**Status:** **FIXED AND PROVEN** — see Verification below.
**Component:** `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs`

### Symptom, as first observed

Every `prometheus kbd` mutation failed:

```
prometheus kbd decision record --id D1-base-ui-over-shadcn …
  Caused by:
    0: client error (Connect)
    1: tcp connect error
    2: Connection refused (os error 61)
```

### How it destroyed work — the composed failure

This is the part that matters. Two individually-survivable facts combined to lose data:

1. Codex archived the four C-00 changes at **05:54** and recorded completion in
   `progress.json` (the file projection) — because `prometheus kbd` writes were
   impossible, that was the *only* place the completion existed.
2. `prometheus kbd migrate --apply` ran at **06:09** and rebuilt `progress.json` from
   canonical runtime state — which had never received those completions.

Result: **C-00 silently reverted to `PENDING`, 0/21**, while all four changes sat
archived on disk. A harness resuming from KBD state would have redone four archives.

### Root cause (verified in code, not inferred)

Mutations need current state to build a `CommandEnvelope` (expected revision + causal
frontier). Fifteen call sites read it with a bare:

```rust
let state = client.status().await?;      // TCP-only, no fallback
```

`ControlClient::status()` speaks HTTP to `127.0.0.1:7892`. But under the managed
configuration the daemon **never binds TCP** — `ai.prometheus.sovereign-sync.plist`
passes only `--mode daemon`, and sovereign-sync serves over a Unix socket. The CLI's
`reqwest` client cannot speak `unix://` (`URL scheme is not allowed`), which the source
itself documents at `kbd.rs:745`.

So `status()` *always* fails, every mutation dies at the **precondition read**, and
`submit_fresh`'s correct `Unreachable → execute_locally` fallback is never reached.

**This was a known bug with an incomplete fix.** Commit `374f313`
*"fix(kbd): commit locally when the control plane cannot adjudicate"* fixed exactly one
call site — the generic `Command` path at line 584 — with a comment naming
*"the exact failure Codex hit."* The other 15 sites were left unfixed.

### Fix

Added `state_or_replay(client, runtime)`, mirroring the proven line-584 pattern: try the
daemon, fall back to `runtime.replay()` — the same journal the daemon itself reads, so
the resulting envelope is identical to the one the daemon would have built.

Applied to all 15 sites:

- 12 bare `client.status().await?` → `state_or_replay(&client, &runtime).await?`
- 2 additional sites (`Pause`, `Cancel`) that were **worse than the rest**: both write a
  local emergency-pause file *before* the read, so a hard failure stranded the operator
  half-applied — paused on disk with no durable record.
- 1 site (`status` display, line 630) intentionally unchanged; it already branches on its
  own fallback.

### Why local fallback is safe here

Not a workaround — it is the designed path, per the source's own reasoning:

- The envelope carries a `command_id` generated once, so a later daemon replay recognises
  the command instead of double-applying it.
- `Runtime::execute_command` checks `state.command_revisions` for that id *before*
  validating the frontier, short-circuiting duplicates.
- Local commit uses the identical journal, signing, validation, and `flock`.
- An adjudicated `Rejected` is still honoured — `may_execute_locally()` returns false —
  so a daemon refusal is never laundered into a local success.

### Verification — **COMPLETE, FIX PROVEN** (2026-08-07)

| # | Check | Result |
|---|---|---|
| 1 | `cargo check -p prometheus-cli` | **PASS**, zero warnings |
| 2 | `cargo build --release` | **PASS** → `c28b2c70716377…` |
| 3 | Install (`cp -f` + `codesign --force --sign -`, per `install_bin`) | installed `8aa2b05fa5895e…`; `prometheus --version` → 1.7.0 (no SIGKILL, so the ad-hoc re-sign is required and worked) |
| 4 | **Re-run the failing case** — `decision record D1` | **PASS** — `"committedLocally": true`, `"remoteStatusUnknown": false`, revision 2 → **3** |
| 5 | D2, D3 | **PASS** — revisions **4**, **5** |
| 6 | Persistence | all three ids present in canonical state: `D1-base-ui-over-shadcn`, `D2-per-surface-scoping`, `D3-vendor-knowme-standard` |
| 7 | **Idempotency** | re-running D1 with the same `command-id` returned revision **5, unchanged** — the duplicate short-circuited instead of double-applying, exactly as `command_revisions` is designed to |
| 8 | **Durability** | `project.loro` grew 14466 → **19236** bytes; a *fresh process* reads `revision 5` — committed to disk, not held in memory |
| 9 | Signing | committed under operator key `ed25519:354034667941…` |
| 10 | No regression | C-00 still `COMPLETE`, phase still 1/21 — the write did not clobber the projection |
| 11 | `prometheus doctor` | **6/6 managed binaries executable, hashed, signed**; picked up the new hash, signature valid |

**Before → after, same command, same machine:**

```
BEFORE:  Caused by: 0: client error (Connect)
                    1: tcp connect error
                    2: Connection refused (os error 61)

AFTER:   "committedLocally": true, "remoteStatusUnknown": false     revision 3
```

The daemon is still Unix-socket-only and TCP :7892 is still unbound — nothing about the
environment changed. The CLI now degrades to the local runtime instead of dying at the
precondition read, which is what `374f313` intended for all call sites.

---

## F-002 — `migrate --apply` silently discards projection state that is ahead of canonical

**Severity:** HIGH — this is the defect that actually destroyed C-00.
**Status:** **FIXED AND PROVEN** (2026-08-07).
**Component:** `substrate/kbd-runtime/src/lib.rs`

### Root cause

`migrate_legacy_ledgers(apply = true)` calls
`write_compatibility_projections_migrating()` unconditionally, regenerating every
`generatedBy: "kbd-runtime"` file from replayed canonical state.

Legacy phases are imported into canonical state only under
`if state.phases.is_empty() && !phases.is_empty()` — i.e. on a **first** migration. On any
later run against populated state, the file completions are read for the summary counters
but never reconciled. Anything recorded only in the projection is overwritten.

It *did* already refuse to overwrite files lacking the `generatedBy` marker. That protects
foreign files, not the runtime's own — which is precisely where harness completions land.

### Fix

Added `projections_ahead_of_canonical(kbd_root, paths, state)` plus two small helpers
(`projection_completed`, `canonical_completed`). Before the projection rewrite,
`migrate --apply` compares each phase's on-disk completed count against the count of
canonical `Change`s in `WorkStatus::Complete`. If any projection is **ahead**, it returns
`RuntimeError::InvalidState` naming the phases and both numbers, pointing at the backup
directory, and telling the operator to reconcile via `prometheus kbd change`/`task` first.

A phase absent from canonical state counts as 0 — the strongest "ahead" signal.

### Verification — the exact scenario that lost the work, replayed

| # | Step | Result |
|---|---|---|
| 1 | `cargo check` (kbd-runtime), `cargo build --release` | **PASS** → installed `4493aa8b2d2ce0…` |
| 2 | **No false positive:** `migrate --check` on consistent state | 47 files, `staleProjections: 0` — unaffected |
| 3 | **The test:** `migrate --apply` with projection at 1 completed, canonical at 0 | **REFUSED** — `refusing to migrate: 1 phase projection(s) record more completed work than canonical state … uar-uiux-full-migration-2026-08: projection records 1 completed, canonical state has 0` |
| 4 | Projection after refusal | **C-00 still `COMPLETE`, 1/21** — the work that was destroyed at 06:09 survives |
| 5 | Reconcile via CLI: `change register` + `transition` ×2 | revisions 6, 7; canonical now reports `C-00: complete` |
| 6 | **Not just a blanket block:** re-run `migrate --apply` | **PROCEEDED**, `staleProjections: 0` |
| 7 | C-00 after the successful migration | **`status: DONE`, `implementation_status: COMPLETE`, 1/21, all 21 changes intact** |

Step 3 and step 7 are the proof: the same command that silently reverted C-00 to PENDING
now refuses when it would lose work, and preserves it when reconciled. Steps 2 and 6
confirm the guard discriminates rather than blocking all migrations.

**Note on the two fixes together.** F-001 made it *possible* for a harness to write
completions into canonical state; F-002 makes it *safe* to migrate when one hasn't. Either
alone leaves the data-loss path open — F-001 without F-002 still loses work from any
harness using the documented projection fallback, and F-002 without F-001 refuses forever
because no harness could ever reconcile.

---

## F-004 — Locally-committed commands never refreshed the projections harnesses read

**Severity:** HIGH — commands succeed while every harness reads stale state.
**Status:** **FIXED AND PROVEN** (2026-08-07).
**Component:** `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs`

**Found by F-001's fix working.** Only once writes started succeeding did it become
visible that they changed nothing a harness could see.

### Symptom

```
$ prometheus kbd revise --exact-next-work "/opsx:new amend-goal4-base-ui-divergence"
Run: … revision 9   Lifecycle: Running   plan revision 4      # canonical: correct

$ jq -c '{revision, exactNextCommand}' .kbd-orchestrator/current-waypoint.json
{"revision":7,"exactNextCommand":"openspec archive a2ui-uar-renderer-on-webcore -y"}
```

Canonical state advanced to revision 9 with the correct next command. The waypoint stayed
at revision 7, still pointing at an **already-completed** archive — which would have sent
the next agent to redo finished work. Precisely the C-00 failure mode again, by a
different route.

### Root cause

When the daemon commits, it rewrites `current-waypoint.json` and the phase
`progress.json` files server-side. `ControlClient::execute_locally` — the fallback path —
called `runtime.execute_command(envelope)`, checked `apply_error`, and returned
`result.state` **without ever writing the projections**. Durable in canonical state,
invisible everywhere else.

### Fix

After a successful local commit, call
`write_compatibility_projections_from_state(&result.state, Utc::now())`, mirroring what
the daemon does. Best-effort: a projection-write failure warns and still returns success,
because the command *is* durably committed — turning that into an error would be worse
than a stale file.

### Verification

| Step | Result |
|---|---|
| Before | waypoint `revision 7`, `exactNextCommand: openspec archive a2ui-uar-renderer-on-webcore -y` (completed); canonical at revision 9 |
| Run `revise` with the fix installed | canonical → revision 10, plan revision 5 |
| After | waypoint **`revision 10`, `planRevision 5`, `exactNextCommand: /opsx:new amend-goal4-base-ui-divergence`** |
| Phase counters | `1/21`, C-00 `DONE`, C-01 `PENDING` — consistent, not clobbered |
| `prometheus doctor` | managed binary manifest valid, new hash `3cf7ff59d73bf6…` |

The gap closed in one command. Waypoint and canonical state now agree.

---

## F-003 — Two pack root documents describe a control plane that no longer exists

**Severity:** LOW (documentation). **Status:** OPEN.

`KBD-HANDOFF.md` and `KBD-RECOVERY-PROBLEM-REPORT.md` (both 2026-08-02) state that
sovereign-sync does not compile, that the launch agent is "intentionally unloaded," and
that health checks take 12s. All are stale:

| Claim (2026-08-02) | Measured 2026-08-07 |
|---|---|
| does not compile; inflight patch pending | patch gone; `ef12667 remove residual voter facade` shipped |
| launch agent unloaded, nothing on :7892 | agent **loaded**, PID 902, healthy |
| `/health` 12s | **p50 2.7ms** |
| `sovereign-sync 1485f964…` / `prometheus 3c95828f…` | both binaries newer |

An agent reading these as current would conclude KBD is unusable and work around it.
They should be dated-and-superseded or archived.

---

## Confirmed working

Recorded so the run produces positive evidence, not only defects:

- **Migration on real legacy state.** 47 progress files, 42 legacy read-only phases,
  0 invalid, 0 alias conflicts, history replayable. `staleProjections` 1 → 0, mode flipped
  legacy → canonical (`revision 2`), all 21 changes registered in correct order, and it
  declined to overwrite 14 historical files it had not authored.
- **Cross-harness handoff.** Codex resumed from `execution.md` + `position-reminder.txt`,
  correctly identified the C-00 blocker I had flagged (`base-ui-foundation` missing its
  spec delta), wrote `specs/frontend-component-primitives/spec.md`, and archived all four
  changes — adding its own `verify-archive-readiness.sh` and `verification-output.txt`.
- **Adversarial review.** Three artifacts reviewed under artifact-only isolation; 16
  CRITICAL findings raised, all independently re-verified. Two were load-bearing errors of
  mine that would have misdirected the plan (a fabricated `+143` regression trend, and a
  16× undercount of `hsl(var())` occurrences). One critic claim was **wrong** and correctly
  rejected on re-measurement (`base-ui-verification` is 0/33, not 0/37) — the loop
  disagrees with critics when the evidence says so, rather than deferring.
