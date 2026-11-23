You are starting the UAR 1.4.0-rc.7 KBD phase. You are running in Codex on an Intel Mac (x86_64-apple-darwin). This is a different machine from the one that ran rc.6. The scope comes from https://github.com/Prometheus-AGS/universal-agent-runtime/issues/542. Your job in this session is to bootstrap the machine, open the successor phase, then run Assess and propose a Plan. Do not implement anything.

## 0. Orient: read these before acting
- `AGENTS.md`, which holds the standing rules. Then `versions.toml`, which is authoritative for pins and architecture, and `.prometheus/gotchas.md`. Pay particular attention to the 2026-11-21 and 2026-11-22 entries: submodules in a fresh checkout, the two model-directory keys, and the eight `sidecar_launch_security` cases.
- `.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/reflection.md`. Read its Delta, Lessons, Closure and Next Phase Seed sections. Also read `closure-packet.md` in the same folder.
- Issue #542. Every item in it is a proposal, not adopted scope.
- State the phase you are in before every step.

## 1. Bootstrap this machine (stop and report on any failure; do not work around it)
1. Clone or update `Prometheus-AGS/universal-agent-runtime` and check out `main`. It must include merge `3a574c3e` (PR #543). Initialize the submodules; `vendor/git/liter-llm` is required for Cargo.
2. Install the Prometheus skill pack, release 1.16.2 or later, with the Codex target:
   `./install.sh --profile skills --targets codex`, then `./install.sh --verify --targets codex --non-interactive`.
   Confirm that `prometheus --version` is 1.15.0 or later, that `prometheus kbd phase close --help` lists `superseded`, and that `prometheus kbd replicas --help` lists `--reprobe`.
3. Register the checkout: `prometheus kbd register .`. Then run `prometheus kbd replicas --json`.
   - If this replica is `readOnly`, run `prometheus kbd replicas --reprobe` from an unrestricted shell.
   - A sandboxed shell can fail the lock probe. That is a known cause, so don't treat the failure as a filesystem fault.
4. Verify the predecessor: `prometheus kbd --path . phase disposition --id uar-uiux-architecture-2026-11 --json` must show `CLOSED_SUPERSEDED`, with closure key `754b54bc…5659`.
   - KBD canonical state may be local to the old machine.
   - If this machine doesn't show that closure, **stop and report**. Do not recreate, replay or re-close the predecessor.

## 2. Open the successor (operator-signed; you never sign)
1. Proposed successor id `uar-rc7-2026-11`, title "UAR 1.4.0-rc.7: remaining UI/UX goals and #388, #520, #524". The operator may change either.
2. Emit the manifest:
   `bash ~/.prometheus/plugins/prometheus-skill-pack/current/shared/scripts/kbd-scope-authority-template.sh start-successor --phase uar-uiux-architecture-2026-11 --successor <id> --title "<title>" --reason "<why, citing #542>" --approver <principal from ~/.config/prometheus/kbd-scope-allowed-signers>`
   Save it under the new phase's `amendments/` folder.
   - If this machine has no allowed-signers entry for the operator, stop and say so.
3. Hand the operator the exact signing command, then wait for them to sign:
   `ssh-keygen -Y sign -f <their key> -n prometheus-kbd-scope-amendment <manifest>`
4. Authorize through `kbd-next-phase` with `--authorize-successor <manifest> --signature <sig>`. Then confirm that the new phase is active.

## 3. Assess, then propose a Plan (stop before Execute)
- Run `kbd-assess` against #542 and the current code on `main`.
  - Recount the open OpenSpec tasks yourself; #542's counts are a starting point.
  - Check that the rc.6 follow-ups still reproduce.
- **One value slice per phase.** This is the main lesson from rc.6. That phase bundled nine goals and could not close. Propose an ordered list of slices with the user outcome for each, and recommend which one this phase should adopt. Move the rest to successor phases.
  - The S2 to S5 proposals are listed in #542.
  - #524 overlaps the config-boot-safety change.
- For the recommended slice:
  - Name each Gherkin scenario, its real production entry point, and its fixture or oracle.
  - Confirm that every scenario can actually run on this Intel Mac before proposing adoption, and name any prerequisite that is missing.
  - Do not adopt or sign a contract. Present it, then stop for the operator.

## Standing rules (from AGENTS.md and the operator; these override defaults)
- **Validation:** BDD/VLBV scenarios only, run once at the boundary on code new since rc.6. No unit-test suites, mutants or repeat sampling.
- **Clean build:** a build must finish with no errors and no warnings, including in the patched crates under `vendor/patches/`. Fix a warning at its source, or annotate it with `#[expect(lint, reason = "...")]`. Never use blanket `allow` or `RUSTFLAGS`.
- **Builds:** use sccache. Agree a build window before a cold `server-full` build; on rc.6 one cold build took 80 minutes and starved other work.
- **GitHub Actions:** deployment only. Never put tests in a workflow. Run `pnpm github-actions-policy:validate` before and after any workflow edit.
- **Authentication and secrets:** no test may use authentication, tokens or credentials. Never read `~/.uar`, `.env` files or environment listings.
- **Running services:** never touch a running UAR instance or another session's processes.
- **Git:** never run `git add -A`; commit by name. Never commit `static/` build output.
- **Permissions:** ask before deleting anything. If a command is denied, report it; do not route around it.
- **Approval required:** signing, publishing, tagging and pushing each need the operator's explicit approval.
- **Reporting:** report each result as PASS, FAIL, BLOCKED or NOT_EXERCISED, with the command you ran and its output. Unverified claims must be labeled unverified.

When you stop, report what you did, the slice you recommend, any blockers, and the exact next action that needs the operator.
